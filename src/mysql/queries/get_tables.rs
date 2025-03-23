use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::{
    core::models::db::{Table, TableColumn},
    mysql::models::mysql_table_column::MySqlTableColumn,
};

pub async fn get_tables(
    pool: &MySqlPool,
    schemas: &[String],
    table_names: &Option<Vec<String>>,
) -> sqlx::Result<Vec<Table>> {
    let tables_argument = table_names.clone().unwrap_or_default().join(",");

    // get all tables from the database
    let query = "
SELECT
    c.TABLE_NAME AS table_name,
    c.COLUMN_NAME AS column_name,
    c.COLUMN_TYPE AS udt_name,
    c.DATA_TYPE AS data_type,
    '' AS table_schema,
    (c.IS_NULLABLE = 'YES') AS is_nullable,
    (c.COLUMN_KEY = 'PRI') AS is_primary_key,
    (c.COLUMN_KEY = 'UNI') AS is_unique,
    kcu.REFERENCED_TABLE_NAME AS foreign_key_table,
    kcu.REFERENCED_COLUMN_NAME AS foreign_key_id,
    NULLIF(c.COLUMN_COMMENT, '') AS column_comment,
    NULLIF(t.TABLE_COMMENT, '') AS table_comment,
    CASE
         WHEN c.COLUMN_DEFAULT IS NOT NULL
              OR c.EXTRA LIKE '%auto_increment%'
              OR c.EXTRA LIKE '%GENERATED%'
         THEN TRUE
         ELSE FALSE
    END AS is_auto_populated,
    0 AS array_depth
FROM
    INFORMATION_SCHEMA.COLUMNS c
LEFT JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
    ON c.TABLE_SCHEMA = kcu.TABLE_SCHEMA
    AND c.TABLE_NAME = kcu.TABLE_NAME
    AND c.COLUMN_NAME = kcu.COLUMN_NAME
    AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
LEFT JOIN INFORMATION_SCHEMA.TABLES t
    ON c.TABLE_SCHEMA = t.TABLE_SCHEMA
    AND c.TABLE_NAME = t.TABLE_NAME
WHERE
    c.TABLE_SCHEMA = DATABASE()
    AND c.TABLE_NAME != '_sqlx_migrations'
ORDER BY
    c.TABLE_NAME,
    c.ORDINAL_POSITION;
".to_string();

    let rows = sqlx::query_as::<sqlx::MySql, MySqlTableColumn>(query.as_str())
        .fetch_all(pool)
        .await?;
    let mut table_map: HashMap<(String, String, Option<String>), Vec<TableColumn>> = HashMap::new();

    for row in rows {
        if let Some(table_names) = table_names {
            if !table_names.contains(&row.table_name) {
                continue;
            }
        }
        table_map
            .entry((
                row.table_name.clone(),
                row.table_schema.clone(),
                row.table_comment.clone(),
            ))
            .or_default()
            .push(TableColumn::from(row));
    }

    let mut tables: Vec<Table> = Vec::new();

    for ((table_name, table_schema, table_comment), columns) in table_map {
        tables.push(Table {
            table_name,
            table_schema: None,
            columns,
            table_comment,
        });
    }

    Ok(tables)
}
