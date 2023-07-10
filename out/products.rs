#[derive(sqlx::FromRow)]
pub struct Products {
 pub id: i64,
 pub created_at: Option<chrono::DateTime<chrono::Utc>>,
 pub label: String,
 pub description: Option<String>,
 pub category: Option<i64>,
}
