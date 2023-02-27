use quote::quote;

pub(crate) struct TypeToTokenHelpers;

impl TypeToTokenHelpers {
    pub(crate) fn optional_literal(s: &Option<impl AsRef<str>>) -> proc_macro2::TokenStream {
        match s {
            Some(s) => {
                let s = s.as_ref();
                quote!(::std::option::Option::Some(#s))
            }
            None => quote!(::std::option::Option::None),
        }
    }

    pub(crate) fn string_literal(s: &Option<impl AsRef<str>>) -> proc_macro2::TokenStream {
        match s {
            Some(s) => {
                let s = s.as_ref();
                quote!(::std::string::ToString::to_string(#s))
            }
            None => quote!(quote!(::std::string::ToString::to_string(""))),
        }
    }

    pub(crate) fn optional_literal_string(s: &Option<impl AsRef<str>>) -> proc_macro2::TokenStream {
        match s {
            Some(s) => {
                let s = s.as_ref();
                quote!(::std::option::Option::Some(::std::string::ToString::to_string(#s)))
            }
            None => quote!(::std::option::Option::None),
        }
    }
}
pub struct ConvertVariableHelpers;

impl ConvertVariableHelpers {
    pub fn underscore_to_camel(s: String) -> String {
        s.split('_').map(|s| s.chars().next().unwrap().to_uppercase().to_string() + &s[1..]).collect()
    }
}
