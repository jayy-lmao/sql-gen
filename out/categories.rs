#[derive(sqlx::FromRow)]
struct Categories {
    id: i64,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    label: String,
}
