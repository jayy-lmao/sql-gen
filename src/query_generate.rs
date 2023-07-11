use crate::{
    models::TableColumn,
    utils::{to_pascal_case, to_snake_case},
};

pub fn generate_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut query_code = String::new();
    query_code.push_str(&format!(
        "use sqlx::{{PgExecutor, Postgres, query_as, query, Result, Error}};\n"
    ));
    query_code.push_str(&format!("use super::{};\n\n", struct_name));
    query_code.push_str(&format!("pub struct {}Set;\n\n", struct_name));
    query_code.push_str(&format!("impl {}Set {{\n", struct_name));

    // Generate query code for SELECT statements
    query_code.push_str(&generate_select_query_code(table_name, rows));
    query_code.push('\n');

    // Generate query code for INSERT statements
    query_code.push_str(&generate_insert_query_code(table_name, rows));
    query_code.push('\n');

    // Generate query code for UPDATE statements
    query_code.push_str(&generate_update_query_code(table_name, rows));
    query_code.push('\n');

    // Generate query code for DELETE statements
    query_code.push_str(&generate_delete_query_code(table_name, rows));
    query_code.push('\n');

    query_code.push_str("}\n");
    query_code
}

fn generate_select_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut select_code = String::new();
    select_code.push_str(&format!(
        "    pub async fn select_all<'e, E: PgExecutor<'e>>(executor: E) -> Result<Vec<{}>, Error> {{\n",
        struct_name
    ));
    select_code.push_str(&format!(
        "        query_as::<_, {}>(\"SELECT * FROM {}\")\n",
        struct_name, table_name
    ));
    select_code.push_str("            .fetch_all(executor)\n");
    select_code.push_str("            .await\n");
    select_code.push_str("    }\n");
    select_code
}

fn generate_insert_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut insert_code = String::new();
    insert_code.push_str(&format!(
        "    pub async fn insert<'e, E: PgExecutor<'e>>(&self, {}: {}, executor: E) -> Result<{}, Error> {{\n",
        to_snake_case(table_name),
        struct_name,
        struct_name
    ));
    insert_code.push_str(&format!(
        "        query_as::<_, {}>(\"INSERT INTO {} ({}) VALUES ({})\")\n",
        struct_name,
        table_name,
        generate_column_list(table_name, rows),
        generate_placeholder_list(table_name, rows)
    ));
    insert_code.push_str(&format!(
        "            {}\n",
        generate_value_list(table_name, rows)
    ));
    insert_code.push_str("            .execute(executor)\n");
    insert_code.push_str("            .await\n");
    insert_code.push_str("    }\n");
    insert_code
}

fn generate_update_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut update_code = String::new();
    update_code.push_str(&format!(
        "    pub async fn update<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<(), Error> {{\n"
    ));
    update_code.push_str(&format!(
        "        query_as::<_, {}>(\"UPDATE {} SET {} WHERE {}\")\n",
        struct_name,
        table_name,
        generate_update_values(table_name, rows),
        generate_update_conditions(table_name, rows)
    ));
    update_code.push_str(&format!(
        "            {}\n",
        generate_value_list(table_name, rows)
    ));
    update_code.push_str(&format!("            .execute(executor)\n"));
    update_code.push_str("    }\n");
    update_code
}

fn generate_delete_query_code(table_name: &str, rows: &[TableColumn]) -> String {
    let mut delete_code = String::new();
    delete_code.push_str(&format!(
        "    pub async fn delete<'e, E: PgExecutor<'e>>(&self, executor: E) -> Result<(), Error> {{\n"
    ));
    delete_code.push_str(&format!(
        "        query(\"DELETE FROM {} WHERE {}\")\n",
        table_name,
        generate_delete_conditions(table_name, rows)
    ));
    delete_code.push_str(&format!("            .execute(executor)\n"));
    delete_code.push_str(&format!("            .map(|_| ())\n"));
    delete_code.push_str("    }\n");
    delete_code
}

fn generate_column_list(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|row| row.table_name == table_name)
        .map(|row| format!("{}", row.column_name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_placeholder_list(table_name: &str, rows: &[TableColumn]) -> String {
    let placeholders = rows
        .iter()
        .filter(|row| row.table_name == table_name)
        .enumerate()
        .map(|(idx, col)| format!("${}", idx + 1))
        .collect::<Vec<String>>()
        .join(", ");
    format!("{}", placeholders)
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
        .filter(|(idx, row)| !row.is_primary_key)
        .map(|(idx, row)| format!("{} = ${}", row.column_name, idx + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_update_conditions(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name)
        .enumerate()
        .filter(|(idx, row)| row.is_primary_key)
        .map(|(idx, row)| format!("{} = {}", row.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}

fn generate_select_by_conditions(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name && r.is_primary_key)
        .enumerate()
        .map(|(idx, row)| format!("{} = {}", row.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}

fn generate_delete_conditions(table_name: &str, rows: &[TableColumn]) -> String {
    rows.iter()
        .filter(|r| r.table_name == table_name && r.is_primary_key)
        .enumerate()
        .map(|(idx, row)| format!("{} = {}", row.column_name, idx + 1))
        .collect::<Vec<String>>()
        .join(" AND ")
}
