use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct MySqlEnumRow {
    pub(crate) schema: String,
    pub(crate) enum_type: String,
    pub(crate) enum_value: String,
    pub(crate) enum_type_comment: Option<String>,
}
