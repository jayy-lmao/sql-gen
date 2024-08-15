use crate::{
    models::{TableColumn, UserDefinedEnums},
    STATE,
};

#[derive(Debug)]
pub(crate) enum DateTimeLib {
    Time,
    Chrono,
}

impl Default for DateTimeLib {
    fn default() -> Self {
        DateTimeLib::Chrono
    }
}

impl DateTimeLib {
    pub(crate) fn date_type(&self) -> &str {
        match self {
            DateTimeLib::Time => "time::Date",
            DateTimeLib::Chrono => "chrono::NaiveDate",
        }
    }
    pub(crate) fn time_type(&self) -> &str {
        match self {
            DateTimeLib::Time => "time::Time",
            DateTimeLib::Chrono => "chrono::NaiveTime",
        }
    }
    pub(crate) fn timestamp_type(&self) -> &str {
        match self {
            DateTimeLib::Time => "time::OffsetDateTime",
            DateTimeLib::Chrono => "chrono::NaiveDateTime",
        }
    }
    pub(crate) fn timestampz_type(&self) -> &str {
        match self {
            DateTimeLib::Time => "time::OffsetDateTime",
            DateTimeLib::Chrono => "chrono::DateTime<chrono::Utc>",
        }
    }
}

impl From<String> for DateTimeLib {
    fn from(value: String) -> Self {
        if value == "chrono" {
            DateTimeLib::Chrono
        } else {
            DateTimeLib::Time
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SqlGenState {
    pub user_defined: Vec<String>,
    pub date_time_lib: DateTimeLib,
}

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

pub fn generate_enum_code(
    enum_name: &str,
    enum_rows: &Vec<UserDefinedEnums>,
    enable_serde: bool,
) -> String {
    let rs_enum_name = to_pascal_case(enum_name);
    let mut enum_code = String::new();

    let serde_derives = if enable_serde {
        ", serde::Serialize, serde::Deserialize"
    } else {
        ""
    };
    enum_code.push_str(&format!(
        "#[derive(sqlx::Type, Debug, Clone, Eq, PartialEq{})]\n",
        serde_derives
    ));
    enum_code.push_str(&format!(r#"#[sqlx(type_name = "{}")]"#, enum_name));
    enum_code.push_str("\n");
    enum_code.push_str(&format!("pub enum {} {{\n", rs_enum_name));

    for row in enum_rows.iter().filter(|e| &e.enum_name == enum_name) {
        enum_code.push_str(&format!(r#"  #[sqlx(rename = "{}")]"#, row.enum_value));
        enum_code.push_str("\n");
        enum_code.push_str(&format!("  {},\n", to_pascal_case(&row.enum_value)))
    }

    enum_code.push_str("}\n");

    enum_code
}

pub fn generate_struct_code(
    table_name: &str,
    rows: &Vec<TableColumn>,
    enable_serde: bool,
) -> String {
    let struct_name = to_pascal_case(table_name);
    let mut struct_code = String::new();

    let serde_derives = if enable_serde {
        ", serde::Serialize, serde::Deserialize"
    } else {
        ""
    };

    struct_code.push_str("#![allow(dead_code)]\n");
    struct_code.push_str("// Generated with sql-gen\n// https://github.com/jayy-lmao/sql-gen\n\n");
    struct_code.push_str(&format!(
        "#[derive(sqlx::FromRow, Debug, Clone{})]\n",
        serde_derives
    ));
    struct_code.push_str(&format!("pub struct {} {{\n", struct_name));

    for row in rows {
        if row.table_name == table_name {
            let column_name = to_snake_case(&row.column_name);
            let mut data_type = convert_data_type(&row.udt_name);
            if row.is_nullable {
                data_type = format!("Option<{}>", data_type);
            }
            struct_code.push_str(&format!(
                "  pub {}: {},\n",
                rust_type_fix(column_name.as_str()),
                data_type
            ));
        }
    }
    struct_code.push_str("}\n");

    struct_code
}

pub fn convert_data_type(data_type: &str) -> String {
    if data_type.to_lowercase().contains("char(") {
        return "String".to_string();
    }
    if data_type.starts_with("_") {
        let array_of_type = convert_data_type(&data_type[1..]);
        let vec_type = format!("Vec<{}>", array_of_type);
        return vec_type;
    }
    let state = STATE.get().unwrap();

    match data_type {
        "bool" | "boolean" => "bool",
        "bytea" => "Vec<u8>", // is this right?
        "char" | "bpchar" | "character" => "String",
        "date" => state.date_time_lib.date_type(),
        "float4" | "real" => "f32",
        "float8" | "double precision" | "numeric" => "f64",
        "int2" | "smallint" | "smallserial" => "i16",
        "int4" | "int" | "serial" => "i32",
        "int8" | "bigint" | "bigserial" => "i64",
        "void" => "()",
        "jsonb" | "json" => "serde_json::Value",
        "text" | "varchar" | "name" | "citext" => "String",
        "geometry" => "String", // when sqlx supports geo types we could change this
        "time" => state.date_time_lib.time_type(),
        "timestamp" => state.date_time_lib.timestamp_type(),
        "timestamptz" => state.date_time_lib.timestampz_type(),
        "interval" => "sqlx::postgres::types::PgInterval",
        "uuid" => "uuid::Uuid",
        _ => {
            if state.user_defined.contains(&data_type.to_string()) {
                return format!("crate::{}", to_pascal_case(data_type));
            } else {
                panic!("Unknown type: {}", data_type)
            }
        }
    }
    .to_string()
}

pub fn convert_data_type_from_pg(data_type: &str) -> String {
    if data_type.contains("Json<") {
        return "jsonb".to_string();
    }
    if data_type.contains("Vec<") {
        let array_type = convert_data_type_from_pg(&data_type[4..data_type.len() - 1]);
        return format!("{}[]", array_type);
    }
    match data_type {
        "i64" => "int8",
        "i32" => "int4",
        "i16" => "int2",
        "String" => "text",
        "serde_json::Value" => "jsonb",
        "chrono::DateTime<chrono::Utc>" => "timestamptz",
        "chrono::NaiveDateTime" => "timestamp",
        "DateTime<Utc>" => "timestamptz",
        "chrono::NaiveDate" => "date",
        "f32" => "float4",
        "f64" => "float8",
        "uuid::Uuid" => "uuid",
        "bool" => "boolean",
        "Vec<u8>" => "bytea", // is this right ?
        _ => panic!("Unknown type: {}", data_type),
    }
    .to_string()
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

pub(crate) fn rust_type_fix(column_name: &str) -> String {
    if column_name == "type" {
        String::from("r#type")
    } else {
        column_name.to_string()
    }
}
