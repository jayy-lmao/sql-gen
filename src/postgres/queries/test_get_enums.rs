use std::error::Error;

use crate::{
    core::models::CustomEnum,
    postgres::queries::{get_enums::get_postgres_enums, test_helper::get_test_pool},
};

#[tokio::test]
async fn test_get_postgres_enums() -> Result<(), Box<dyn Error>> {
    let pool = get_test_pool().await;

    sqlx::query(" DROP TYPE IF EXISTS mood CASCADE;")
        .execute(&pool)
        .await?;

    sqlx::query(" CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');")
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
