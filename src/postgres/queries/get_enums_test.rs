use std::error::Error;

use sqlx::PgPool;

use crate::{core::models::db::CustomEnum, postgres::queries::get_enums::get_postgres_enums};

#[sqlx::test]
#[setup_db_macros::setup_pg_db]
async fn test_get_postgres_enums(pool: PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query("DROP TYPE IF EXISTS mood CASCADE;")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');")
        .execute(&pool)
        .await?;

    let enums = get_postgres_enums(&pool).await?;

    let expected = vec![CustomEnum {
        name: "mood".to_string(),
        schema: "public".to_string(),
        variants: vec!["sad".to_string(), "ok".to_string(), "happy".to_string()],
    }];

    assert_eq!(enums, expected);

    Ok(())
}
