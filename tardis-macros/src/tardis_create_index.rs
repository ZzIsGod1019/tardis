use crate::macro_helpers::helpers::{default_doc, ConvertVariableHelpers, TypeToTokenHelpers};
use darling::FromField;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::token::{Comma, Dot};
use syn::{Attribute, Data, Error, Fields, Result};

#[derive(FromField, Debug, Clone)]
#[darling(attributes(index))]
struct CreateIndexMeta {
    ident: Option<Ident>,
    #[darling(default = "default_index_id")]
    index_id: String,
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    primary: bool,
    #[darling(default)]
    unique: bool,
    #[darling(default)]
    full_text: bool,
    #[darling(default)]
    if_not_exists: bool,
    /// # Index Types
    /// Index type value needs to be one of the following: \
    /// BTree: `index_type = "BTree"` \
    /// FullText: `index_type = "FullText"` \
    /// Gin: `index_type = "Gin"` \
    /// Hash: `index_type = "Hash"` \
    /// Custom: `index_type = "Custom(you custom type)"` \
    ///
    /// example for custom:
    /// ```ignore
    /// #[derive(Clone, Debug, DeriveEntityModel, TardisCreateIndex)]
    /// #[sea_orm(table_name = "examples")]
    /// pub struct Model {
    ///     #[sea_orm(primary_key)]
    ///     pub id: String,
    ///     #[index(index_id = "index_id_1", index_type = "Custom(Test)")]
    ///     pub custom_index_col: String,
    /// }
    ///
    /// //impl Iden for Test ...
    /// ```
    #[darling(default)]
    index_type: Option<String>,
}
fn default_index_id() -> String {
    format!("{}_id", nanoid::nanoid!(4))
}
pub(crate) fn create_index(ident: Ident, data: Data, _atr: impl IntoIterator<Item = Attribute>) -> Result<TokenStream> {
    if ident != "Model" {
        panic!("Struct name must be Model");
    }
    match data {
        Data::Struct(data_struct) => {
            let col_token = create_col_token_statement(data_struct.fields)?;
            let doc = default_doc();
            Ok(quote! {#doc
                fn tardis_create_index_statement() -> Vec<::tardis::db::sea_orm::sea_query::IndexCreateStatement> {
                vec![
                    #col_token
                    ]
            }})
        }
        Data::Enum(_) => Err(Error::new(ident.span(), "enum is not support!")),
        Data::Union(_) => Err(Error::new(ident.span(), "union is not support!")),
    }
}

fn create_col_token_statement(fields: Fields) -> Result<TokenStream> {
    let mut statement: Punctuated<TokenStream, Comma> = Punctuated::new();
    let mut map: HashMap<String, Box<Vec<CreateIndexMeta>>> = HashMap::new();
    for field in fields {
        for attr in field.attrs.clone() {
            if let Some(ident) = attr.path().get_ident() {
                if ident == "index" {
                    let field_create_index_meta: CreateIndexMeta = match CreateIndexMeta::from_field(&field) {
                        Ok(field) => field,
                        Err(err) => {
                            return Ok(err.write_errors());
                        }
                    };
                    if let Some(vec) = map.get_mut(&field_create_index_meta.index_id) {
                        vec.push(field_create_index_meta)
                    } else {
                        map.insert(field_create_index_meta.index_id.clone(), Box::new(vec![field_create_index_meta]));
                    }
                    // out of attr for loop, into next field
                    break;
                }
            }
        }
    }
    for k in map.keys() {
        statement.push(single_create_index_statement(map.get(k).unwrap())?);
    }
    Ok(quote! {#statement})
}
fn single_create_index_statement(index_metas: &Vec<CreateIndexMeta>) -> Result<TokenStream> {
    let mut create_statement: Punctuated<TokenStream, Dot> = Punctuated::new();
    let mut column: Punctuated<TokenStream, Dot> = Punctuated::new();
    let mut name = None;
    let mut primary = false;
    let mut unique = false;
    let mut full_text = false;
    let mut if_not_exists = false;
    let mut index_type = (None, Span::call_site());

    for index_meta in index_metas {
        if let Some(ident) = index_meta.ident.clone() {
            let ident = Ident::new(ConvertVariableHelpers::underscore_to_camel(ident.to_string()).as_ref(), ident.span());
            //add Column
            column.push(quote!(col(Column::#ident)));

            if name.is_none() && index_meta.name.is_some() {
                name = index_meta.name.clone();
            }
            if index_type.0.is_none() && index_meta.index_type.is_some() {
                index_type = (index_meta.index_type.clone(), ident.span());
            }
            if index_meta.primary {
                primary = true;
            }
            if index_meta.unique {
                unique = true;
            }
            if index_meta.full_text {
                full_text = true;
            }
            if index_meta.if_not_exists {
                if_not_exists = true;
            }
        }
    }

    if primary {
        create_statement.push(quote!(primary()))
    }
    if unique {
        create_statement.push(quote!(unique()))
    }
    if full_text {
        create_statement.push(quote!(full_text()))
    }
    if if_not_exists {
        create_statement.push(quote!(if_not_exists()))
    }

    if let (Some(index_type), span) = index_type {
        index_type_map(&index_type, span, &mut create_statement)?;
    }

    let all_statement = if create_statement.is_empty() {
        quote! {#column}
    } else {
        quote! {#column.#create_statement}
    };
    if column.is_empty() {
        Ok(quote! {})
    } else {
        let name = if let Some(name) = name {
            TypeToTokenHelpers::str_literal(&Some(name))
        } else {
            let nano_id = &nanoid::nanoid!(4);
            quote! {&format!("idx-{}_{}", Entity.table_name(),#nano_id)}
        };
        Ok(quote! {::tardis::db::sea_orm::sea_query::Index::create().name(#name).table(Entity).#all_statement.to_owned()})
    }
}
/// Map index type method
fn index_type_map(index_type: &str, span: Span, create_statement: &mut Punctuated<TokenStream, Dot>) -> Result<()> {
    #[cfg(feature = "reldb-postgres")]
    match index_type {
        "BTree" | "b_tree" => {
            create_statement.push(quote!(index_type(::tardis::db::sea_orm::sea_query::IndexType::BTree)));
        }
        "FullText" | "full_text" => {
            create_statement.push(quote!(full_text()));
        }
        "Gin" | "GIN" | "gin" => {
            create_statement.push(quote!(full_text()));
        }
        "Hash" | "hash" => {
            create_statement.push(quote!(index_type(::tardis::db::sea_orm::sea_query::IndexType::Hash)));
        }
        _ => {
            if index_type.starts_with("Custom") || index_type.starts_with("custom") {
                if let Some(paren) = index_type.find('(') {
                    let custom_index_type = &index_type[paren + 1..index_type.len() - 1];
                    let custom_index_type = Ident::new(custom_index_type, span);
                    let custom_statement = quote!(#custom_index_type{});
                    create_statement
                        .push(quote!(index_type(::tardis::db::sea_orm::sea_query::IndexType::Custom(tardis::db::sea_orm::sea_query::types::SeaRc::new(#custom_statement)))));
                    return Ok(());
                };
            }
            return Err(Error::new(span, "not supported index_type!"));
        }
    }
    Ok(())
}
