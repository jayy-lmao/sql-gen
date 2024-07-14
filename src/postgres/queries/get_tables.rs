use std::collections::HashMap;

use sqlx::PgPool;

use crate::{
    core::models::{Table, TableColumn},
    postgres::models::postgres_table_column::PostgresTableColumn,
};

pub async fn get_tables(
    pool: &PgPool,
    schemas: Vec<&str>,
    table_names: Option<&Vec<String>>,
) -> sqlx::Result<Vec<Table>> {
    // get all tables from the database
    let query = "
SELECT
    c.table_name,
    c.column_name,
    c.udt_name,
    c.data_type,
    c.table_schema,
    c.is_nullable = 'YES' AS is_nullable,
    CASE
        WHEN kcu.column_name IS NOT NULL THEN TRUE
        ELSE FALSE
    END AS is_primary_key,
    CASE
        WHEN u.column_name IS NOT NULL THEN TRUE
        ELSE FALSE
    END AS is_unique,
    f.foreign_table_name AS foreign_key_table,
    f.foreign_column_name AS foreign_key_id
FROM
    information_schema.columns c
LEFT JOIN
    (
        SELECT
            tc.table_schema,
            tc.table_name,
            kcu.column_name
        FROM
            information_schema.table_constraints AS tc
        JOIN
            information_schema.key_column_usage AS kcu
        ON
            tc.constraint_schema = kcu.constraint_schema
            AND tc.constraint_name = kcu.constraint_name
        WHERE
            tc.constraint_type = 'PRIMARY KEY'
    ) AS kcu
ON
    c.table_schema = kcu.table_schema
    AND c.table_name = kcu.table_name
    AND c.column_name = kcu.column_name
LEFT JOIN
    (
        SELECT
            tc.table_schema,
            tc.table_name,
            kcu.column_name
        FROM
            information_schema.table_constraints AS tc
        JOIN
            information_schema.key_column_usage AS kcu
        ON
            tc.constraint_schema = kcu.constraint_schema
            AND tc.constraint_name = kcu.constraint_name
        WHERE
            tc.constraint_type = 'UNIQUE'
    ) AS u
ON
    c.table_schema = u.table_schema
    AND c.table_name = u.table_name
    AND c.column_name = u.column_name
LEFT JOIN
    (
        SELECT
            tc.table_schema,
            tc.table_name,
            kcu.column_name,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
        JOIN
            information_schema.key_column_usage AS kcu
        ON
            tc.constraint_schema = kcu.constraint_schema
            AND tc.constraint_name = kcu.constraint_name
        JOIN
            information_schema.constraint_column_usage AS ccu
        ON
            ccu.constraint_schema = tc.constraint_schema
            AND ccu.constraint_name = tc.constraint_name
        WHERE
            tc.constraint_type = 'FOREIGN KEY'
    ) AS f
ON
    c.table_schema = f.table_schema
    AND c.table_name = f.table_name
    AND c.column_name = f.column_name
WHERE
    c.table_schema = ANY($1)
    AND c.table_name != '_sqlx_migrations'
    AND
    ($2 IS NULL OR c.table_name = ANY($2))
ORDER BY
    c.table_name,
    c.ordinal_position;

";

    let rows = sqlx::query_as::<_, PostgresTableColumn>(query)
        .bind(schemas)
        .bind(table_names)
        .fetch_all(pool)
        .await?;
    let mut table_map: HashMap<(String, String), Vec<TableColumn>> = HashMap::new();

    for row in rows {
        table_map
            .entry((row.table_name.clone(), row.table_schema.clone()))
            .or_default()
            .push(TableColumn::from(row));
    }

    let mut tables: Vec<Table> = Vec::new();

    for ((table_name, table_schema), columns) in table_map {
        tables.push(Table {
            table_name,
            table_schema,
            columns,
        });
    }

    Ok(tables)
}
