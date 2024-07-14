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
select
    c.table_name,
    c.column_name,
    c.udt_name,
    c.data_type,
    c.table_schema,
    c.is_nullable = 'yes' as is_nullable,
    case
        when k.column_name is not null then true
        else false
    end as is_primary_key,
    case
        when u.column_name is not null then true
        else false
    end as is_unique,
    f.foreign_table_name as foreign_key_table,
    f.foreign_column_name as foreign_key_id
from
    information_schema.columns c
left join
    information_schema.key_column_usage k on
    c.table_schema = k.table_schema and
    c.table_name = k.table_name and
    c.column_name = k.column_name and
    k.constraint_name in (
        select
            constraint_name
        from
            information_schema.table_constraints
        where
            constraint_type = 'primary key'
    )
left join (
    select
        tc.table_schema,
        tc.table_name,
        kcu.column_name
    from
        information_schema.table_constraints as tc
    join
        information_schema.key_column_usage as kcu on
        tc.constraint_schema = kcu.constraint_schema and
        tc.constraint_name = kcu.constraint_name
    where
        tc.constraint_type = 'unique'
) as u on
    c.table_schema = u.table_schema and
    c.table_name = u.table_name and
    c.column_name = u.column_name
left join (
    select
        tc.table_schema,
        tc.table_name,
        kcu.column_name,
        ccu.table_name as foreign_table_name,
        ccu.column_name as foreign_column_name
    from
        information_schema.table_constraints as tc
    join
        information_schema.key_column_usage as kcu on
        tc.constraint_schema = kcu.constraint_schema and
        tc.constraint_name = kcu.constraint_name
    join
        information_schema.constraint_column_usage as ccu on
        ccu.constraint_schema = tc.constraint_schema and
        ccu.constraint_name = tc.constraint_name
    where
        tc.constraint_type = 'foreign key'
) as f on
    c.table_schema = f.table_schema and
    c.table_name = f.table_name and
    c.column_name = f.column_name
where
    c.table_schema = any($1)
    and c.table_name != '_sqlx_migrations'
    and
    ($2 is null or c.table_name = any($2))
order by
    c.table_name,
    c.ordinal_position;
";

    let rows = sqlx::query_as::<_, PostgresTableColumn>(query)
        .bind(schemas)
        .bind(table_names)
        .fetch_all(pool)
        .await?;
    let mut table_map: HashMap<String, Vec<TableColumn>> = HashMap::new();

    for row in rows {
        table_map
            .entry(row.table_name.clone())
            .or_default()
            .push(TableColumn::from(row));
    }

    let mut tables: Vec<Table> = Vec::new();

    for (table_name, columns) in table_map {
        tables.push(Table {
            table_name,
            columns,
        });
    }

    Ok(tables)
}
