pub fn convert_data_type(udt_type: &str) -> Option<String> {
    if udt_type.to_lowercase().contains("char(") {
        return Some("String".to_string());
    }

    if let Some(stripped_vec_type) = udt_type.strip_prefix("_") {
        let array_of_type = convert_data_type(stripped_vec_type);
        return array_of_type;
    }

    match udt_type {
        "bool" | "boolean" => Some("bool".to_string()),
        "bytea" => Some("u8".to_string()), // is this right?
        "char" | "bpchar" | "character" => Some("String".to_string()),
        "date" => Some("chrono::NaiveDate".to_string()),
        "float4" | "real" => Some("f32".to_string()),
        "float8" | "double precision" => Some("f64".to_string()),
        "int2" | "smallint" | "smallserial" => Some("i16".to_string()),
        "int4" | "int" | "serial" => Some("i32".to_string()),
        "int8" | "bigint" | "bigserial" => Some("i64".to_string()),
        "void" => Some("()".to_string()),
        "jsonb" | "json" => Some("serde_json::Value".to_string()),
        "text" | "varchar" | "name" | "citext" => Some("String".to_string()),
        "time" => Some("chrono::NaiveTime".to_string()),
        "timestamp" => Some("chrono::NaiveDateTime".to_string()),
        "timestamptz" => Some("chrono::DateTime<chrono::Utc>".to_string()),
        "uuid" => Some("uuid::Uuid".to_string()),
        _ => None,
    }
}
