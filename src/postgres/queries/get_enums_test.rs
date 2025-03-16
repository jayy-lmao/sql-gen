use std::error::Error;

use crate::{
    core::models::db::{CustomEnum, CustomEnumVariant},
    postgres::{queries::get_enums::get_postgres_enums, test_helper::setup_pg_db},
};

#[tokio::test]
async fn test_get_postgres_enums() -> Result<(), Box<dyn Error>> {
    let pool = setup_pg_db().await;

    sqlx::query("DROP TYPE IF EXISTS mood CASCADE;")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');")
        .execute(&pool)
        .await?;

    let enums = get_postgres_enums(&pool).await?;

    let expected = vec![CustomEnum {
        name: "mood".to_string(),
        type_name: Some("mood".to_string()),
        schema: Some("public".to_string()),
        variants: vec![
            CustomEnumVariant {
                name: "sad".to_string(),
            },
            CustomEnumVariant {
                name: "ok".to_string(),
            },
            CustomEnumVariant {
                name: "happy".to_string(),
            },
        ],
        comments: None,
        ..Default::default()
    }];

    assert_eq!(enums, expected);

    Ok(())
}

#[tokio::test]
async fn test_get_postgres_enums_with_comments() -> Result<(), Box<dyn Error>> {
    let pool = setup_pg_db().await;
    // Clean up any existing type
    sqlx::query("DROP TYPE IF EXISTS weather CASCADE;")
        .execute(&pool)
        .await?;

    // Create the enum type
    sqlx::query("CREATE TYPE weather AS ENUM ('rainy', 'cloudy', 'sunny');")
        .execute(&pool)
        .await?;

    // Add a comment to the enum type
    sqlx::query("COMMENT ON TYPE weather IS 'This enum represents different weather';")
        .execute(&pool)
        .await?;

    // Retrieve the enum definitions including comments
    let enums = get_postgres_enums(&pool).await?;

    // Define the expected result including the comment on the type
    let expected = vec![CustomEnum {
        name: "weather".to_string(),
        type_name: Some("weather".to_string()),
        schema: Some("public".to_string()),
        variants: vec![
            CustomEnumVariant {
                name: "rainy".to_string(),
            },
            CustomEnumVariant {
                name: "cloudy".to_string(),
            },
            CustomEnumVariant {
                name: "sunny".to_string(),
            },
        ],
        comments: Some("This enum represents different weather".to_string()),
        ..Default::default()
    }];

    pretty_assertions::assert_eq!(enums, expected);

    Ok(())
}
