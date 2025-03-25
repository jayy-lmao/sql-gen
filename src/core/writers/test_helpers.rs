use std::str::FromStr;

use super::helpers::pretty_print_tokenstream;

pub fn tokenstream_from_string(input: &str) -> Result<proc_macro2::TokenStream, String> {
    proc_macro2::TokenStream::from_str(input)
        .map_err(|err| syn::Error::new(proc_macro2::Span::call_site(), err).to_string())
}

pub fn format_rust_content_string(input: &str) -> String {
    pretty_print_tokenstream(tokenstream_from_string(input).expect("Could not parse"))
}
