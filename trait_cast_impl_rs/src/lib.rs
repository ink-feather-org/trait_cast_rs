//! Proc-macro automating the implementation of `trait_cast_rs::TraitcastableAny`.
//!
//! See `make_trait_castable` for more details.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
  parse::{self, Parse, ParseStream},
  parse_macro_input,
  punctuated::Punctuated,
  Error, ItemEnum, ItemStruct, Token, TypePath,
};

/// Parses a list of `TypePath`s separated by commas.
struct TraitCastTargets {
  targets: Vec<TypePath>,
}

impl Parse for TraitCastTargets {
  fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
    let targets: Vec<TypePath> = Punctuated::<TypePath, Token![,]>::parse_terminated(input)?
      .into_iter()
      .collect();
    Ok(TraitCastTargets { targets })
  }
}

impl quote::ToTokens for TraitCastTargets {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    let vars = &self.targets;
    tokens.extend(quote!(#(#vars),*));
  }
}

/// Attribute macro implementing `TraitcastableAny` for a struct, enum or union.
///
/// Use the arguments to specify all possible target Traits for witch trait objects are
/// supposed to be downcastable from a dyn `TraitcastableAny`.
///
/// Example:
/// ```no_build
///   extern crate trait_cast_rs;
///
///   use trait_cast_rs::{make_trait_castable, TraitcastTarget, TraitcastTo, TraitcastableAny};
///
///
///   #[make_trait_castable(Print)]
///   struct Source(i32);
///
///   trait Print {
///     fn print(&self);
///   }
///   impl Print for Source {
///     fn print(&self) {
///       println!("{}", self.0)
///     }
///   }
///
///   fn main() {
///     let source = Box::new(Source(5));
///     let castable: Box<dyn TraitcastableAny> = source;
///     let x: &dyn Print = castable.downcast_ref().unwrap();
///     x.print();
///   }
/// ```
#[proc_macro_attribute]
pub fn make_trait_castable(args: TokenStream1, input: TokenStream1) -> TokenStream1 {
  // Convert the input to a TokenStream2
  let input = TokenStream2::from(input);

  let trait_cast_targets = parse_macro_input!(args as TraitCastTargets);

  // First, try to parse the input as a struct
  let input_struct = syn::parse2::<ItemStruct>(input.clone());
  let mut source_ident = input_struct.map(|item_struct| item_struct.ident);

  // Maybe it's an enum
  if source_ident.is_err() {
    let input_enum = syn::parse2::<ItemEnum>(input.clone());
    source_ident = input_enum.map(|item_enum| item_enum.ident);
  }

  if let Err(err) = source_ident {
    let mut custom_error_message = Error::new(err.span(), "Expected a struct or enum");
    custom_error_message.combine(err);
    return custom_error_message.to_compile_error().into();
  }

  let source_ident = source_ident.unwrap();

  TokenStream1::from(quote!(
    #input
    ::trait_cast_rs::make_trait_castable_decl! {
    #source_ident => (#trait_cast_targets)
  }))
}
