use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Error, Ident, Item, Token};

struct Args {
  vars: Vec<Ident>,
}

impl Parse for Args {
  fn parse(input: ParseStream) -> Result<Self> {
    let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
    let vars = vars.into_iter().collect::<Vec<_>>();
    for i in 0..vars.len() {
      let first_trait = &vars[i];
      for it in 0..i {
        let second_trait = &vars[it];
        if first_trait == second_trait {
          let mut error = Error::new(
            second_trait.span(),
            format!("Trait `{}` used multiple times", second_trait),
          );
          error.combine(Error::new(
            first_trait.span(),
            format!("Trait `{}` first used here", first_trait),
          ));
          return Err(error);
        }
      }
    }
    Ok(Args { vars })
  }
}

fn gen_mapping_funcs(item_name: &Ident, args: &Args) -> TokenStream2 {
  let to_dyn_funcs = args
    .vars
    .iter()
    .map(|ident| {
      let to_dyn_name = format_ident!("__internal_to_dyn_{ident}");
      quote_spanned!(ident.span() =>
        pub fn #to_dyn_name(input: &dyn Traitcastable) -> Option<&dyn #ident> {
          let any: &dyn Any = input;
          any.downcast_ref::<Self>().map(|selv| selv as &dyn #ident)
        }
      )
    })
    .collect::<TokenStream2>();
  let expanded = quote!(
    impl #item_name {
      #to_dyn_funcs
    }
  );
  expanded
}
fn gen_target_func(item_name: &Ident, args: &Args) -> TokenStream2 {
  let targets = args
    .vars
    .iter()
    .map(|ident| {
      let to_dyn_name = format_ident!("__internal_to_dyn_{ident}");
      quote_spanned!(ident.span() =>
        TraitcastTarget::new(
          std::any::TypeId::of::<dyn #ident>(),
          std::mem::transmute(HybridPet::#to_dyn_name as fn(_) -> _),
        ),
      )
    })
    .collect::<TokenStream2>();
  let expanded = quote!(
    impl ::trait_cast_rs::Traitcastable for #item_name {
      fn traitcastable_from(&self) -> &'static [TraitcastTarget] {
        const TARGETS: &'static [TraitcastTarget] = unsafe {
          &[
            #targets
          ]
        };
        TARGETS
      }
    }
  );
  expanded
}

#[proc_macro_attribute]
pub fn make_trait_castable(args: TokenStream, input: TokenStream) -> TokenStream {
  // TODO: Invoke macro_rules for hygiene?
  let input = parse_macro_input!(input as Item);
  let item_name = match &input {
    Item::Enum(item_enum) => &item_enum.ident,
    Item::Struct(item_struct) => &item_struct.ident,
    _ => {
      return TokenStream::from(
        Error::new(
          input.span(),
          "The `make_trait_castable` attribute can only be applied to enums or structs",
        )
        .to_compile_error(),
      )
    },
  };

  // Parse the list of variables the user wanted to print.
  let args = parse_macro_input!(args as Args);

  // Use a syntax tree traversal to transform the function body.
  // let output = args.fold_item_fn(input);

  // Hand the resulting function body back to the compiler.
  // TokenStream::from(quote!(#output))
  let mapping_funcs = gen_mapping_funcs(item_name, &args);
  let target_func = gen_target_func(item_name, &args);
  let output = TokenStream::from(quote!(
    #input
    #mapping_funcs
    #target_func
  ));
  output
}
