use syn::File;

pub fn pretty_print_tokenstream(ts: proc_macro2::TokenStream) -> String {
    match syn::parse2::<File>(ts.clone()) {
        Ok(file) => prettyplease::unparse(&file).to_string(),
        Err(err) => format!("Failed to parse TokenStream: {err}. Stream was {ts}"),
    }
}
