#![deny(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]
//! Proc-macro automating the implementation of `trait_cast_rs::TraitcastableAny`.
//!
//! See `make_trait_castable` for more details.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::quote;
use venial::{parse_declaration, Declaration, Error};

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
  let args = TokenStream::from(args);
  let input = match parse_declaration(input.into()) {
    Ok(Declaration::Function(fun)) => {
      let mut error = Error::new_at_span(
        fun.name.span(),
        "Can not implement `Traitcast` for functions",
      );
      error.combine(Error::new_at_span(
        fun.name.span(),
        "Expected a struct, enum or union definition",
      ));
      return error.to_compile_error().into();
    },
    Ok(input) => input,
    Err(error) => {
      return error.to_compile_error().into();
    },
  };
  let item_name = input.name();

  TokenStream1::from(quote!(
    #input
    ::trait_cast_rs::make_trait_castable_decl! {
    #item_name => (#args)
  }))
}
