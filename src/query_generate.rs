use crate::{
    models::TableColumn,
    utils::{convert_data_type, rust_type_fix, to_pascal_case, to_snake_case},
};

pub fn generate_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let schema_prefix = match rows
        .iter()
        .find(|&r| r.table_name == table_name)
        .map(|r| r.table_schema.as_str())
    {
        None => "".to_string(),
        Some("public") => "".to_string(), // Default Schema
        Some(schema) => format!("\"{}\".", schema),
    };
    let schema_name = &schema_prefix;

    let mut query_code = String::new();
    query_code.push_str("#![allow(dead_code)]\n");

    query_code.push_str("// Generated with sql-gen\n// https://github.com/jayy-lmao/sql-gen\n\n");
    query_code.push_str(&format!(
        "use sqlx::{{query, query_as, PgExecutor, Result}};\n"
    ));
    query_code.push_str(&format!("use super::{};\n\n", struct_name));
    query_code.push_str(&format!("pub struct {}Set;\n\n", struct_name));
    query_code.push_str(&format!("impl {}Set {{\n", struct_name));

    // Generate query code for SELECT statements
    query_code.push_str(&generate_select_query_code(table_name, schema_name, &rows));
    query_code.push('\n');

    // Generate query code for SELECT BY PK statements
    query_code.push_str(&generate_select_by_pk_query_code(
        table_name,
        schema_name,
        &rows,
    ));
    query_code.push('\n');

    // Generate query code for SELECT MANY BY PK statements
    query_code.push_str(&generate_select_many_by_pks_query_code(
        table_name,
        schema_name,
        &rows,
    ));
    query_code.push('\n');

    // Generate query code for SELECT BY PK Optional statements
    query_code.push_str(&generate_select_by_pk_query_code_optional(
        table_name,
        schema_name,
        &rows,
    ));
    query_code.push('\n');

    query_code.push_str(&generate_unique_query_code(table_name, schema_name, &rows));
    query_code.push('\n');

    query_code.push_str(&generate_select_all_fk_queries(
        table_name,
        schema_name,
        &rows,
    ));
    query_code.push('\n');

    // Generate query code for INSERT statements
    query_code.push_str(&generate_insert_query_code(table_name, schema_name, &rows));
    query_code.push('\n');

    // Generate query code for UPDATE statements
    query_code.push_str(&generate_update_query_code(table_name, schema_name, &rows));
    query_code.push('\n');

    // Generate query code for DELETE statements
    query_code.push_str(&generate_delete_query_code(table_name, schema_name, &rows));
    query_code.push('\n');

    query_code.push_str("}\n");
    query_code
}

fn generate_select_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    select_code.push_str(&format!(
        "    pub async fn all<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<Vec<{}>> {{\n",
        struct_name
    ));
    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\"\"#)\n",
        struct_name, schema_name, table_name
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
        return String::from("");
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| {
            format!(
                "{}: {}",
                rust_type_fix(r.column_name.as_str()),
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn by_{}<'e, E: PgExecutor<'e>>(&self, executor: E, {}) -> Result<{}> {{\n",
        pkey_name, fn_args, struct_name
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
        .map(|r| {
            format!(
                "            .bind({})\n",
                rust_type_fix(r.column_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join("");

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {}\"#)\n",
        struct_name, schema_name, table_name, condition
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
        return String::from("");
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_primary_key && r.table_name == table_name)
        .map(|r| {
            format!(
                "{}_list: Vec<{}>",
                rust_type_fix(r.column_name.as_str()),
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn many_by_{}_list<'e, E: PgExecutor<'e>>(&self, executor: E, {}) -> Result<Vec<{}>> {{\n",
        pkey_name, fn_args, struct_name
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
        .map(|r| {
            format!(
                "            .bind({}_list)\n",
                rust_type_fix(r.column_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join("");

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {}\"#)\n",
        struct_name, schema_name, table_name, condition
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
                rust_type_fix(r.column_name.as_str()),
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    select_code.push_str(&format!(
        "    pub async fn by_{}_optional<'e, E: PgExecutor<'e>>(&self, executor: E, {}) -> Result<Option<{}>> {{\n",
        pkey_name,
        fn_args,
        struct_name
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
        .map(|r| {
            format!(
                "            .bind({})\n",
                rust_type_fix(r.column_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join("");

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {}\"#)\n",
        struct_name, schema_name, table_name, condition
    ));
    select_code.push_str(&bind);
    select_code.push_str("            .fetch_optional(executor)\n");

    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_unique_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let mut code = String::new();
    let mut unique_rows = std::collections::HashMap::<(&str, &str), &TableColumn>::new();
    for row in rows
        .iter()
        .filter(|r| r.table_name.as_str() == table_name && r.is_unique)
    {
        unique_rows.insert((&row.table_name, &row.column_name), row);
    }
    let unique_users_vec: Vec<&TableColumn> = unique_rows.into_values().collect();
    for row in unique_users_vec.iter() {
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
        return String::from("");
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| {
            format!(
                "{}: {}",
                rust_type_fix(r.column_name.as_str()),
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>();

    if fn_args.is_empty() {
        return String::from("");
    }
    let fn_args = fn_args.join(", ");

    select_code.push_str(&format!(
        "    pub async fn unique_by_{}<'e, E: PgExecutor<'e>>(&self, executor: E, {}) -> Result<{}> {{\n",
        unique_name, fn_args, struct_name
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
        .map(|r| {
            format!(
                "            .bind({})\n",
                rust_type_fix(r.column_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join("");

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {}\"#)\n",
        struct_name, schema_name, table_name, condition
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
        return String::from("");
    }

    let fn_args = rows
        .iter()
        .filter(|r| r.is_unique && r.table_name == table_name && r.column_name == unique_name)
        .map(|r| {
            let rs_type = convert_data_type(r.udt_name.as_str());
            if rs_type.starts_with("Vec<") {
                None
            } else {
                Some(format!(
                    "{}_list: Vec<{}>",
                    rust_type_fix(r.column_name.as_str()),
                    rs_type
                ))
            }
        })
        .collect::<Option<Vec<String>>>();

    if fn_args.is_none() {
        return String::from("");
    }
    let fn_args = fn_args.unwrap().join(", ");

    select_code.push_str(&format!(
        "    pub async fn unique_many_by_{}_list<'e, E: PgExecutor<'e>>(&self, executor: E, {}) -> Result<Vec<{}>> {{\n",
        unique_name, fn_args, struct_name
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
        .map(|r| {
            format!(
                "            .bind({}_list)\n",
                rust_type_fix(r.column_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join("");

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {}\"#)\n",
        struct_name, schema_name, table_name, condition
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
                rust_type_fix(r.column_name.as_str()),
                convert_data_type(r.udt_name.as_str())
            )
        })
        .collect::<Vec<String>>();

    if fn_args.is_empty() {
        return String::from("");
    }
    let fn_args = fn_args.join(", ");

    select_code.push_str(&format!(
        "    pub async fn unique_by_{}_optional<'e, E: PgExecutor<'e>>(&self, executor: E, {}) -> Result<Option<{}>> {{\n",
        unique_name,
        fn_args,
        struct_name
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
        .map(|r| {
            format!(
                "            .bind({})\n",
                rust_type_fix(r.column_name.as_str())
            )
        })
        .collect::<Vec<String>>()
        .join("");

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {}\"#)\n",
        struct_name, schema_name, table_name, condition
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
                    &table_name,
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
        to_snake_case(column_name),
        to_snake_case(foreign_row_table_name),
        foreign_row_column_name,
        data_type,
        struct_name
    ));

    select_code.push_str(&format!(
        "        query_as::<_, {}>(r#\"SELECT * FROM {}\"{}\" WHERE {} = $1\"#)\n",
        struct_name, schema_name, table_name, column_name
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
    update_code.push_str(&format!("            .fetch_one(executor)\n"));
    update_code.push_str(&format!("            .await\n"));
    update_code.push_str("    }\n");
    update_code
}

fn generate_delete_query_code(table_name: &str, schema_name: &str, rows: &[TableColumn]) -> String {
    let mut delete_code = String::new();
    delete_code.push_str(&format!(
        "    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<()> {{\n"
    ));
    delete_code.push_str(&format!(
        "        query(r#\"DELETE FROM {}\"{}\" WHERE {}\"#)\n",
        schema_name,
        table_name,
        generate_delete_conditions(table_name, rows)
    ));
    delete_code.push_str(&format!("            .execute(executor)\n"));
    delete_code.push_str(&format!("            .await\n"));
    delete_code.push_str(&format!("            .map(|_| ())\n"));
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
    format!("{}", placeholders)
}

fn generate_value_list(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|row| row.table_name == table_name)
        .map(|row| {
            let column_name = rust_type_fix(row.column_name.as_str());
            format!(
                ".bind({}.{})",
                to_snake_case(&row.table_name),
                to_snake_case(&column_name)
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
