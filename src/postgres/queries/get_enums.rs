use std::collections::HashMap;

use sqlx::PgPool;

use crate::{core::models::CustomEnum, postgres::models::postgres_enum::PostgresEnumRow};

pub async fn get_postgres_enums(pool: &PgPool) -> Result<Vec<CustomEnum>, sqlx::Error> {
    let query = r#"
        SELECT
            n.nspname AS schema,
            t.typname AS enum_type,
            e.enumlabel AS enum_value
        FROM
            pg_type t
            JOIN pg_enum e ON t.oid = e.enumtypid
            JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
        WHERE
            n.nspname NOT IN ('pg_catalog', 'information_schema')
        ORDER BY
            schema, enum_type, e.enumsortorder;
    "#;

    let rows: Vec<PostgresEnumRow> = sqlx::query_as::<_, PostgresEnumRow>(query)
        .fetch_all(pool)
        .await?;

    let mut enum_map: HashMap<(String, String), Vec<String>> = HashMap::new();

    for row in rows {
        enum_map
            .entry((row.schema.clone(), row.enum_type.clone()))
            .or_default()
            .push(row.enum_value);
    }

    let mut enums: Vec<CustomEnum> = Vec::new();

    for ((schema, name), variants) in enum_map {
        enums.push(CustomEnum {
            name,
            schema,
            variants,
        });
    }

    Ok(enums)
}
