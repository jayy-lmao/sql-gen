// In your proc-macro crate (e.g. setup_db_macro)

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn setup_pg_db(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    // Insert a call to your database setup function at the beginning of the function.
    let setup_stmt: syn::Stmt = syn::parse_quote! {
        {
            // Create a runtime to block on the async initialization.
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(crate::postgres::test_helper::initialize_database());
        }
    };
    input.block.stmts.insert(0, setup_stmt);

    TokenStream::from(quote! {
        #input
    })
}
