#[derive(sqlx::FromRow)]
struct Products {
    id: i64,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    label: String,
    description: Option<String>,
    category: Option<i64>,
}
