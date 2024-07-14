use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct PostgresEnumRow {
    pub(crate) schema: String,
    pub(crate) enum_type: String,
    pub(crate) enum_value: String,
}
