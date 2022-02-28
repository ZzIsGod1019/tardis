use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

use crate::domain;

pub async fn init() -> TardisResult<()> {
    TardisFuns::reldb().create_table_from_entity(domain::todos::Entity, TardisFuns::reldb().conn()).await?;
    Ok(())
}
