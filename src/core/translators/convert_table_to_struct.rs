use convert_case::{Case, Casing};
use pluralizer::pluralize;

use crate::core::models::{db::Table, rust::RustDbSetStruct};

use super::{convert_column_to_field::convert_column_to_field, models::TableToStructOptions};


pub fn convert_table_to_struct(table: Table, options: TableToStructOptions) -> RustDbSetStruct {
    let table_name_pascal_case = table.table_name.clone().to_case(Case::Pascal);
    let table_name_singular = pluralize(&table_name_pascal_case, 1, false);

    let maybe_override = options.override_name.clone();
    //println!("OPTIONS: {:#?}", options);

    let struct_name = maybe_override.unwrap_or(table_name_singular);
    let table_name = table.table_name.clone();
    let fields = table
        .columns
        .iter()
        .filter_map(|c| { 

            let column_override = options.column_overrides.get(&c.column_name).cloned();
            let type_override = options.type_overrides.get(&c.udt_name).cloned();
            let column_to_field_options = column_override.or(type_override).unwrap_or_default();
            let field = convert_column_to_field(c, column_to_field_options  );
            if field.is_none() {
                println!("WARNING: field {} in table {} has no user-defined type or recommended type for {}", c.column_name,&table.table_name,c.udt_name)
            }

            field

        })
        .collect();

    RustDbSetStruct {
        struct_name,
        table_name: Some(table_name),
        fields,
    }
}

