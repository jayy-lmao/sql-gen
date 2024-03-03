use crate::models::TableColumn;

pub(crate) fn to_snake_case(input: &str) -> String {
    let mut output = String::new();
    let mut prev_is_uppercase = false;

    for c in input.chars() {
        if c.is_ascii_uppercase() {
            if !output.is_empty() && !prev_is_uppercase {
                output.push('_');
            }
            output.extend(c.to_lowercase());
            prev_is_uppercase = true;
        } else {
            output.push(c);
            prev_is_uppercase = false;
        }
    }

    output
}

pub fn generate_struct_code(table_name: &str, rows: &Vec<TableColumn>) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut struct_code = String::new();

    struct_code.push_str("#![allow(dead_code)]\n");
    struct_code.push_str("// Generated with sql-gen\n// https://github.com/jayy-lmao/sql-gen\n\n");
    struct_code.push_str("#[derive(sqlx::FromRow, Debug)]\n");
    struct_code.push_str(&format!("pub struct {} {{\n", struct_name));

    for row in rows {
        if row.table_name == table_name {
            let column_name = to_snake_case(&row.column_name);
            let mut data_type = convert_data_type(&row.udt_name);
            let optional_type = format!("Option<{}>", data_type);
            if row.is_nullable {
                data_type = optional_type.as_str();
            }

            struct_code.push_str(&format!("  pub {}: {},\n", column_name, data_type));
        }
    }
    struct_code.push_str("}\n");

    struct_code
}

pub fn convert_data_type(data_type: &str) -> &str {
    match data_type {
        "int8" => "i64",
        "int4" => "i32",
        "int2" => "i16",
        "text" => "String",
        "varchar" => "String",
        "jsonb" => "sqlx::Json",
        "timestamptz" => "chrono::DateTime<chrono::Utc>",
        "date" => "chrono::NaiveDate",
        "float4" => "f32",
        "float8" => "f64",
        "uuid" => "uuid::Uuid",
        "boolean" => "bool",
        "bool" => "bool",
        "bytea" => "Vec<u8>", // is this right?
        _ => panic!("Unknown type: {}", data_type),
    }
}

pub fn convert_data_type_from_pg(data_type: &str) -> &str {
    match data_type {
        "i64" => "int8",
        "i32" => "int4",
        "i16" => "int2",
        "String" => "text",
        "sqlx::Json" => "jsonb",
        "chrono::DateTime<chrono::Utc>" => "timestamptz",
        "DateTime<Utc>" => "timestamptz",
        "chrono::NaiveDate" => "date",
        "f32" => "float4",
        "f64" => "float8",
        "uuid::Uuid" => "uuid",
        "bool" => "boolean",
        "Vec<u8>" => "bytea", // is this right ?
        _ => panic!("Unknown type: {}", data_type),
    }
}

fn generate_query_code(_row: &TableColumn) -> String {
    // ... (implementation of generate_query_code)
    // query_code
    todo!()
}

pub fn parse_struct_fields(struct_code: &str) -> Vec<(String, String, bool)> {
    let lines = struct_code.lines();
    let mut fields = Vec::new();

    for line in lines {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with("pub") {
            continue;
        }

        let parts: Vec<&str> = trimmed_line.split(": ").collect();
        if parts.len() != 2 {
            continue;
        }

        let field = parts[0].trim().trim_start_matches("pub").trim();
        //let data_type_optional = parts[1].trim().trim_end_matches(",").trim();
        let mut is_nullable = false;

        let data_type = if parts[1].trim().starts_with("Option") {
            is_nullable = true;
            parts[1]
                .trim()
                .trim_start_matches("Option<")
                .trim_end_matches(">,")
        } else {
            parts[1].trim().trim_end_matches(',')
        };

        fields.push((field.to_owned(), data_type.to_owned(), is_nullable));
    }

    fields
}

#[cfg(test)]
mod tests {
    // ... (unit tests can be defined here)
}

pub fn to_pascal_case(input: &str) -> String {
    let mut output = String::new();
    let mut capitalize_next = true;

    for c in input.chars() {
        if c.is_ascii_alphanumeric() {
            if capitalize_next {
                output.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                output.push(c);
            }
        } else {
            capitalize_next = true;
        }
    }

    output
}
