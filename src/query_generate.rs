use crate::{
    models::TableColumn,
    utils::{convert_data_type, to_pascal_case, to_snake_case},
};

pub fn generate_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let schema_prefix = match rows
        .iter()
        .find(|&r| r.table_name == table_name)
        .map(|r| r.table_schema.as_str())
    {
        None => String::new(),
        Some("public") => String::new(), // Default Schema
        Some(schema) => format!("\"{schema}\"."),
    };
    let schema_name = &schema_prefix;

    let mut query_code = String::new();
    query_code.push_str("#![allow(dead_code)]\n");

    query_code.push_str("// Generated with sql-gen\n// https://github.com/jayy-lmao/sql-gen\n\n");
    query_code.push_str("use sqlx::{query, query_as, PgExecutor, Result};\n");
    query_code.push_str(&format!("use super::{struct_name};\n\n"));
    query_code.push_str(&format!("pub struct {struct_name}Set;\n\n"));
    query_code.push_str(&format!("impl {struct_name}Set {{\n"));

    // Generate query code for SELECT statements
    query_code.push_str(&generate_select_query_code(table_name, schema_name, rows));
    query_code.push('\n');

    // Generate query code for SELECT BY PK statements
    query_code.push_str(&generate_select_by_pk_query_code(
        table_name,
        schema_name,
        rows,
    ));
    query_code.push('\n');

    // Generate query code for SELECT MANY BY PK statements
    query_code.push_str(&generate_select_many_by_pks_query_code(
        table_name,
        schema_name,
        rows,
    ));
    query_code.push('\n');

    // Generate query code for SELECT BY PK Optional statements
    query_code.push_str(&generate_select_by_pk_query_code_optional(
        table_name,
        schema_name,
        rows,
    ));
    query_code.push('\n');

    query_code.push_str(&generate_unique_query_code(table_name, schema_name, rows));
    query_code.push('\n');

    query_code.push_str(&generate_select_all_fk_queries(
        table_name,
        schema_name,
        rows,
    ));
    query_code.push('\n');

    // Generate query code for INSERT statements
    query_code.push_str(&generate_insert_query_code(table_name, schema_name, rows));
    query_code.push('\n');

    // Generate query code for UPDATE statements
    query_code.push_str(&generate_update_query_code(table_name, schema_name, rows));
    query_code.push('\n');

    // Generate query code for DELETE statements
    query_code.push_str(&generate_delete_query_code(table_name, schema_name, rows));
    query_code.push('\n');

    query_code.push_str("}\n");
    query_code
}

fn generate_select_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    select_code.push_str(&format!(
        "    pub async fn all<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<Vec<{struct_name}>> {{\n"
    ));
    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\"\"#)\n"
    ));
    select_code.push_str("            .fetch_all(executor)\n");
    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_select_by_pk_query_code(
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    let pkey_name = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| r.column_name.as_str())
        .collect::<Vec<&str>>()
        .join("_and_");

    // There were no pk rows
    if pkey_name.is_empty() {
        return String::new();
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| {
            format!(
                "{}: {}",
                r.column_name,
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn by_{pkey_name}<'e, E: PgExecutor<'e>>(&self, executor: E, {fn_args}) -> Result<{struct_name}> {{\n"
    ));

    let condition = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .enumerate()
        .map(|(idx, r)| format!("\"{}\" = ${}", r.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let bind = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| format!("            .bind({})\n", r.column_name))
        .collect::<String>();

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {condition}\"#)\n"
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_one(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_select_many_by_pks_query_code(
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    let pkey_name = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| r.column_name.as_str())
        .collect::<Vec<&str>>()
        .join("_and_");

    // There were no pk rows
    if pkey_name.is_empty() {
        return String::new();
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| {
            format!(
                "{}_list: Vec<{}>",
                r.column_name,
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn many_by_{pkey_name}_list<'e, E: PgExecutor<'e>>(&self, executor: E, {fn_args}) -> Result<Vec<{struct_name}>> {{\n"
    ));

    let condition = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .enumerate()
        .map(|(idx, r)| format!("\"{}\" = ANY(${})", r.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let bind = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| format!("            .bind({}_list)\n", r.column_name))
        .collect::<String>();

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {condition}\"#)\n"
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_all(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_select_by_pk_query_code_optional(
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    let pkey_name = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| r.column_name.as_str())
        .collect::<Vec<&str>>()
        .join("_and_");

    let fn_args = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| {
            format!(
                "{}: {}",
                r.column_name,
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn by_{pkey_name}_optional<'e, E: PgExecutor<'e>>(&self, executor: E, {fn_args}) -> Result<Option<{struct_name}>> {{\n"
    ));

    let condition = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .enumerate()
        .map(|(idx, r)| format!("\"{}\" = ${}", r.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let bind = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| format!("            .bind({})\n", r.column_name))
        .collect::<String>();

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {condition}\"#)\n"
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_optional(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_unique_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let mut code = String::new();
    for row in rows.iter().filter(|r| r.is_unique) {
        code.push_str(
            generate_select_by_unique_query_code(&row.column_name, table_name, schema_name, rows)
                .as_str(),
        );
        code.push('\n');
        code.push_str(
            generate_select_many_by_uniques_query_code(
                &row.column_name,
                table_name,
                schema_name,
                rows,
            )
            .as_str(),
        );
        code.push('\n');
        code.push_str(
            generate_select_by_unique_query_code_optional(
                &row.column_name,
                table_name,
                schema_name,
                rows,
            )
            .as_str(),
        );
        code.push('\n');
    }
    code
}

fn generate_select_by_unique_query_code(
    unique_name: &str,
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();

    // There were no unique rows
    if unique_name.is_empty() {
        return String::new();
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| {
            format!(
                "{}: {}",
                r.column_name,
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn by_{unique_name}<'e, E: PgExecutor<'e>>(&self, executor: E, {fn_args}) -> Result<{struct_name}> {{\n"
    ));

    let condition = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .enumerate()
        .map(|(idx, r)| format!("\"{}\" = ${}", r.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let bind = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| format!("            .bind({})\n", r.column_name))
        .collect::<String>();

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {condition}\"#)\n"
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_one(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_select_many_by_uniques_query_code(
    unique_name: &str,
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();

    // There were no unique rows
    if unique_name.is_empty() {
        return String::new();
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| {
            format!(
                "{}_list: Vec<{}>",
                r.column_name,
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn many_by_{unique_name}_list<'e, E: PgExecutor<'e>>(&self, executor: E, {fn_args}) -> Result<Vec<{struct_name}>> {{\n"
    ));

    let condition = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .enumerate()
        .map(|(idx, r)| format!("\"{}\" = ANY(${})", r.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let bind = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| format!("            .bind({}_list)\n", r.column_name))
        .collect::<String>();

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {condition}\"#)\n"
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_all(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_select_by_unique_query_code_optional(
    unique_name: &str,
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();

    let fn_args = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| {
            format!(
                "{}: {}",
                r.column_name,
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn by_{unique_name}_optional<'e, E: PgExecutor<'e>>(&self, executor: E, {fn_args}) -> Result<Option<{struct_name}>> {{\n"
    ));

    let condition = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .enumerate()
        .map(|(idx, r)| format!("\"{}\" = ${}", r.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ");

    let bind = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| format!("            .bind({})\n", r.column_name))
        .collect::<String>();

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {condition}\"#)\n"
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_optional(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_select_all_fk_queries(
    table_name: &str,
    schema_name: &str,
    rows: &[TableColumn],
) -> String {
    let mut select_code = String::new();

    for row in rows
        .iter()
        .filter(|r| r.foreign_key_table.is_some() && r.table_name == table_name)
    {
        if let (Some(row_foreign_table), Some(row_foreign_id)) =
            (&row.foreign_key_table, &row.foreign_key_id)
        {
            let matching_row = rows
                .iter()
                .find(|r| &r.table_name == row_foreign_table && &r.column_name == row_foreign_id);

            if let Some(foreign_row) = matching_row {
                let fk_query = generate_select_by_fk_query_code(
                    table_name,
                    &row.column_name,
                    schema_name,
                    &foreign_row.table_name,
                    &foreign_row.column_name,
                    &foreign_row.udt_name,
                );
                select_code.push_str(&fk_query);
            }
        }
    }

    select_code
}

fn generate_select_by_fk_query_code(
    table_name: &str,
    column_name: &str,
    schema_name: &str,
    foreign_row_table_name: &str,
    foreign_row_column_name: &str,
    foreign_udt_type: &str,
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    let data_type = convert_data_type(foreign_udt_type);

    select_code.push_str(&format!(
        "    pub async fn all_by_{}_{}<'e, E: PgExecutor<'e>>(executor: E, {}_{}: {}) -> Result<Vec<{}>> {{\n",
        to_snake_case(foreign_row_table_name),
        foreign_row_column_name,
        to_snake_case(foreign_row_table_name),
        foreign_row_column_name,
        data_type,
        struct_name
    ));

    select_code.push_str(&format!(
        "        query_as::<_, {struct_name}>(r#\"SELECT * FROM {schema_name}\"{table_name}\" WHERE {column_name} = $1\"#)\n"
    ));
    select_code.push_str(&format!(
        "            .bind({}_{})\n",
        to_snake_case(foreign_row_table_name),
        foreign_row_column_name,
    ));
    select_code.push_str("            .fetch_all(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_insert_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut insert_code = String::new();
    insert_code.push_str(&format!(
        "    pub async fn insert<'e, E: PgExecutor<'e>>(&self, executor: E, {}: {}) -> Result<{}> {{\n",
        to_snake_case(table_name),
        struct_name,
        struct_name
    ));
    insert_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *;\"#)\n",
        struct_name,
        table_name,
        generate_column_list(table_name, rows),
        generate_placeholder_list(table_name, rows)
    ));
    insert_code.push_str(&format!(
        "            {}\n",
        generate_value_list(table_name, rows)
    ));
    insert_code.push_str("            .fetch_one(executor)\n");
    insert_code.push_str("            .await\n");
    insert_code.push_str("    }\n");
    insert_code
}

fn generate_update_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut update_code = String::new();
    update_code.push_str(&format!(
        "    pub async fn update<'e, E: PgExecutor<'e>>(&self, executor: E, {}: {}) -> Result<{}> {{\n",
        to_snake_case(table_name),
        struct_name,
        struct_name,
    ));
    update_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"UPDATE \"{}\" SET {} WHERE {} RETURNING *;\"#)\n",
        struct_name,
        table_name,
        generate_update_values(table_name, rows),
        generate_update_conditions(table_name, rows)
    ));
    update_code.push_str(&format!(
        "            {}\n",
        generate_value_list(table_name, rows)
    ));
    update_code.push_str("            .fetch_one(executor)\n");
    update_code.push_str("            .await\n");
    update_code.push_str("    }\n");
    update_code
}

fn generate_delete_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let mut delete_code = String::new();
    delete_code.push_str("    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<()> {\n");
    delete_code.push_str(&format!(
        "        query(r#\"DELETE FROM {}\"{}\" WHERE {}\"#)\n",
        schema_name,
        table_name,
        generate_delete_conditions(table_name, rows)
    ));
    delete_code.push_str("            .execute(executor)\n");
    delete_code.push_str("            .await\n");
    delete_code.push_str("            .map(|_| ())\n");
    delete_code.push_str("    }\n");
    delete_code
}

fn generate_column_list(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|row| row.table_name == table_name)
        .map(|row| format!("\"{}\"", row.column_name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_placeholder_list(table_name: &str, rows: &[TableColumn]) -> String {
    let placeholders = rows
        .iter()
        .filter(|row| row.table_name == table_name)
        .enumerate()
        .map(|(idx, _col)| format!("${}", idx + 1))
        .collect::<Vec<String>>()
        .join(", ");
    placeholders.to_string()
}

fn generate_value_list(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|row| row.table_name == table_name)
        .map(|row| {
            format!(
                ".bind({}.{})",
                to_snake_case(&row.table_name),
                to_snake_case(&row.column_name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n            ")
}

fn generate_update_values(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name)
        .enumerate()
        .filter(|(_idx, row)| !row.is_primary_key)
        .map(|(idx, row)| format!("\"{}\" = ${}", row.column_name, idx + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_update_conditions(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name)
        .enumerate()
        .filter(|(_idx, row)| row.is_primary_key)
        .map(|(idx, row)| format!("\"{}\" = {}", row.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}

fn generate_select_by_conditions(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name && r.is_primary_key)
        .enumerate()
        .map(|(idx, row)| format!("\"{}\" = {}", row.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}

fn generate_delete_conditions(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name && r.is_primary_key)
        .enumerate()
        .map(|(idx, row)| format!("\"{}\" = {}", row.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}
