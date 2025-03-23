use super::{convert_column_to_field::convert_column_to_field, models::CodegenOptions};
use crate::{core::models::{
    db::Table,
    rust::{dbset_attribute_with_table_name, RustDbSetStruct},
}, Mode};
use convert_case::{Case, Casing};
use pluralizer::pluralize;

pub fn convert_tables_to_struct(
    tables: Vec<Table>,
    options: &CodegenOptions,
) -> Vec<RustDbSetStruct> {
    tables
        .into_iter()
        .map(|table| convert_table_to_struct(table, options))
        .collect()
}

pub fn convert_table_to_struct(table: Table, options: &CodegenOptions) -> RustDbSetStruct {
    let table_name_pascal_case = table.table_name.clone().to_case(Case::Pascal);
    let table_name_singular = pluralize(&table_name_pascal_case, 1, false);

    let maybe_override = options.override_name.get(&table.table_name);

    let struct_name = maybe_override.unwrap_or(&table_name_singular);
    let table_name = table.table_name.clone();
    let fields = table
        .columns
        .iter()
        .filter_map(|c| { 
            let key = &(table.table_name.clone(), c.column_name.clone());

            let column_override = options
                .table_column_overrides
                .get(key)
                .or(options.column_overrides.get(&c.column_name)).cloned();

            let type_override = options.type_overrides.get(&c.udt_name).cloned();

            let mut column_to_field_options = column_override.or(type_override).unwrap_or_default();
            column_to_field_options.mode = options.mode;
            let field = convert_column_to_field(c, column_to_field_options);
            if field.is_none() {
                println!("WARNING: field {} in table {} has no user-defined type or recommended type for {}", c.column_name,&table.table_name,c.udt_name)
            }

            field

        })
        .collect();

    RustDbSetStruct {
        name: struct_name.to_string(),
        attributes: if options.mode == Mode::Dbset { vec![
            dbset_attribute_with_table_name(table_name)
        ]} else {vec![]},
        fields,
        derives: options.struct_derives.clone(),
        comment: table.table_comment.clone(),
    }
}
