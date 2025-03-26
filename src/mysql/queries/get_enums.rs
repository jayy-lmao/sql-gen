use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::{
    core::models::db::{CustomEnum, CustomEnumVariant},
    mysql::models::mysql_enum::MySqlEnumRow,
};

pub async fn get_mysql_enums(pool: &MySqlPool) -> Result<Vec<CustomEnum>, sqlx::Error> {
    let query = r#"
WITH RECURSIVE enum_split AS (
  -- Initial row: extract the full list of enum values from COLUMN_TYPE
  SELECT
    CAST(c.TABLE_SCHEMA AS CHAR) AS `schema`,
    CONCAT(c.TABLE_NAME, '.', c.COLUMN_NAME) AS enum_type,
    c.COLUMN_TYPE,
    CAST(c.COLUMN_COMMENT AS CHAR) AS enum_type_comment,
    -- Remove the leading "enum(" and trailing ")" then extract the first value.
    CAST(TRIM(BOTH '\'' FROM SUBSTRING_INDEX(SUBSTRING(c.COLUMN_TYPE, 6, CHAR_LENGTH(c.COLUMN_TYPE) - 6 - 1), ',', 1)) AS CHAR) AS enum_value,
    CASE 
      WHEN LOCATE(',', SUBSTRING(c.COLUMN_TYPE, 6, CHAR_LENGTH(c.COLUMN_TYPE) - 6 - 1)) > 0 
      THEN TRIM(LEADING ' ' FROM SUBSTRING(
            SUBSTRING(c.COLUMN_TYPE, 6, CHAR_LENGTH(c.COLUMN_TYPE) - 6 - 1),
            LOCATE(',', SUBSTRING(c.COLUMN_TYPE, 6, CHAR_LENGTH(c.COLUMN_TYPE) - 6 - 1)) + 1))
      ELSE NULL
    END AS rest
  FROM INFORMATION_SCHEMA.COLUMNS c
  WHERE 
      c.DATA_TYPE = 'enum'
    AND c.TABLE_SCHEMA = DATABASE()

  UNION ALL
  
  -- Recursive part: split off the next value from the remainder string.
  SELECT
    `schema`,
    enum_type,
    COLUMN_TYPE,
    enum_type_comment,
    TRIM(BOTH '\'' FROM SUBSTRING_INDEX(rest, ',', 1)) AS enum_value,
    CASE 
      WHEN LOCATE(',', rest) > 0 
      THEN TRIM(LEADING ' ' FROM SUBSTRING(rest, LOCATE(',', rest) + 1))
      ELSE NULL
    END AS rest
  FROM enum_split
  WHERE rest IS NOT NULL
)
SELECT 
  `schema`,
  enum_type,
  enum_value,
  NULLIF(enum_type_comment, '') AS enum_type_comment
FROM enum_split
ORDER BY `schema`, enum_type, enum_value;

    "#;

    let rows: Vec<MySqlEnumRow> = sqlx::query_as::<_, MySqlEnumRow>(query)
        .fetch_all(pool)
        .await?;

    let mut enum_map: HashMap<(String, String, Option<String>), Vec<String>> = HashMap::new();

    for row in rows {
        enum_map
            .entry((
                row.schema.clone(),
                row.enum_type.clone(),
                row.enum_type_comment,
            ))
            .or_default()
            .push(row.enum_value);
    }

    let mut enums: Vec<CustomEnum> = Vec::new();

    for ((_schema, name, enum_comment), variants) in enum_map {
        enums.push(CustomEnum {
            name: name.split(".").nth(1).unwrap().to_string(),
            type_name: None,
            child_of_table: name.split(".").next().map(|s| s.to_string()),
            schema: None,
            comments: enum_comment,
            variants: variants
                .into_iter()
                .map(|v| CustomEnumVariant { name: v })
                .collect(),
        });
    }

    Ok(enums)
}
