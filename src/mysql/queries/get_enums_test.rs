use crate::{
    core::models::db::{CustomEnum, CustomEnumVariant},
    mysql::{queries::get_enums::get_mysql_enums, test_helper::setup_mysql_db},
};
use pretty_assertions::assert_eq;
use std::error::Error;

#[tokio::test]
async fn test_get_mysql_enums() -> Result<(), Box<dyn Error>> {
    let pool = setup_mysql_db().await;

    sqlx::query("DROP TABLE IF EXISTS test_table;")
        .execute(&pool)
        .await?;

    sqlx::query(
        "
        CREATE TABLE test_table (
          id INT AUTO_INCREMENT PRIMARY KEY,
          mood ENUM('sad', 'ok', 'happy') NOT NULL
        );
    ",
    )
    .execute(&pool)
    .await?;
    let enums = get_mysql_enums(&pool).await?;

    let expected = vec![CustomEnum {
        name: "mood".to_string(),
        child_of_table: Some("test_table".to_string()),
        schema: None,
        variants: vec![
            CustomEnumVariant {
                name: "happy".to_string(),
            },
            CustomEnumVariant {
                name: "ok".to_string(),
            },
            CustomEnumVariant {
                name: "sad".to_string(),
            },
        ],
        comments: None,
        ..Default::default()
    }];

    assert_eq!(enums, expected);

    Ok(())
}

#[tokio::test]
async fn test_get_mysql_enums_with_comments() -> Result<(), Box<dyn Error>> {
    let pool = setup_mysql_db().await;
    // Clean up any existing type
    // Drop any existing table that uses the enum.
    sqlx::query("DROP TABLE IF EXISTS test_weather;")
        .execute(&pool)
        .await?;

    // Create a table with a 'weather' column defined as an ENUM with a comment.
    sqlx::query("
    CREATE TABLE test_weather (
      id INT AUTO_INCREMENT PRIMARY KEY,
      weather ENUM('rainy', 'cloudy', 'sunny') NOT NULL COMMENT 'This enum represents different weather'
    );
")
    .execute(&pool)
    .await?;

    // Retrieve the enum definitions including comments
    let enums = get_mysql_enums(&pool).await?;

    // Define the expected result including the comment on the type
    let expected = vec![CustomEnum {
        name: "weather".to_string(),
        child_of_table: Some("test_weather".to_string()),
        schema: None,
        variants: vec![
            CustomEnumVariant {
                name: "cloudy".to_string(),
            },
            CustomEnumVariant {
                name: "rainy".to_string(),
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
