pub fn convert_data_type(udt_type: &str) -> Option<String> {
    if udt_type.to_lowercase().contains("char(") {
        return Some("String".to_string());
    }

    if let Some(stripped_vec_type) = udt_type.strip_prefix("_") {
        let array_of_type = convert_data_type(stripped_vec_type);
        return array_of_type;
    }

    match udt_type {
        // Boolean: MySQL treats TINYINT(1), BOOLEAN and BOOL as booleans.
        "bool" | "boolean" | "tinyint(1)" => Some("bool".to_string()),

        // Numeric types
        "tinyint unsigned" => Some("u8".to_string()),
        "tinyint" => Some("i8".to_string()),
        "smallint unsigned" => Some("u16".to_string()),
        "smallint" => Some("i16".to_string()),
        "int unsigned" => Some("u32".to_string()),
        "int" => Some("i32".to_string()),
        "bigint unsigned" => Some("u64".to_string()),
        "bigint" => Some("i64".to_string()),
        "float" => Some("f32".to_string()),
        "double" => Some("f64".to_string()),

        // String types: VARCHAR, CHAR and TEXT map to Rust’s String.
        "varchar" | "char" | "text" => Some("String".to_string()),

        // Binary types: VARBINARY, BINARY and BLOB map to Vec<u8>
        "varbinary" | "binary" | "blob" => Some("Vec<u8>".to_string()),

        // Date and time types
        "date" => Some("chrono::NaiveDate".to_string()),
        "datetime" => Some("chrono::NaiveDateTime".to_string()),
        "timestamp" => Some("chrono::DateTime<chrono::Utc>".to_string()),
        "time" => Some("chrono::NaiveTime".to_string()),

        // Decimal type
        "decimal" => Some("rust_decimal::Decimal".to_string()),

        // UUID type: MySQL often stores UUIDs as BINARY(16)
        "uuid" => Some("uuid::Uuid".to_string()),

        // JSON type: maps to a serde_json value.
        "json" => Some("serde_json::JsonValue".to_string()),

        _ => None,
    }
}
