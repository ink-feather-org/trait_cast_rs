use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use venial::{parse_declaration, Declaration, Error};

struct Args {
  vars: Vec<Ident>,
}

fn parse_args(args: TokenStream2) -> Result<Args, Error> {
  // Parse the list of variables the user wanted to print.
  let mut vars: Vec<Ident> = vec![];
  let mut ts = args.into_iter();
  while let Some(tt) = ts.next() {
    match tt {
      proc_macro2::TokenTree::Ident(id) => {
        for old_id in &vars {
          if old_id == &id {
            let mut error =
              Error::new_at_span(id.span(), format!("Trait `{}` used multiple times", id));
            error.combine(Error::new_at_span(
              old_id.span(),
              format!("Trait `{}` first used here", old_id),
            ));
            return Err(error);
          }
        }
        vars.push(id);
        let next_tt = ts.next();
        if let Some(tt) = next_tt {
          match tt {
            TokenTree::Punct(punc) if punc.as_char() == ',' => {},
            _ => return Err(Error::new_at_tokens(tt, "Expected ',' or ')' ")),
          }
        }
      },
      _ => return Err(Error::new_at_tokens(tt, "Expected Identifier")),
    }
  }
  Ok(Args { vars })
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
  let args = match parse_args(args.into()) {
    Ok(args) => args,
    Err(err) => return err.to_compile_error().into(),
  };

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

  let mapping_funcs = gen_mapping_funcs(&item_name, &args);
  let target_func = gen_target_func(&item_name, &args);
  let output = TokenStream::from(quote!(
    #input
    #mapping_funcs
    #target_func
  ));
  output
}
