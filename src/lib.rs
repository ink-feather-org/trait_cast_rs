pub mod trait_cast;

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn make_trait_castable(input: TokenStream, annoteated_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[cfg(test)]
mod test;
