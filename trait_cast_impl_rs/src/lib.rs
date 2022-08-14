#![deny(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]
//! Proc macro automating the implementation of `trait_cast_rs::Traitcastable`.
//!
//! See `make_trait_castable` for more details.

use std::{
  fmt::{self, Display, Formatter},
  iter::Peekable,
};

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use venial::{parse_declaration, Declaration, Error};

struct Type {
  fully_qualified: bool, // if it starts with ::
  path: Vec<Ident>,
  generics: Vec<Type>,
}
impl Type {
  fn is_punct(tt: &TokenTree, c: char) -> bool {
    if let TokenTree::Punct(p) = tt && p.as_char() == c {true} else {false}
  }
  fn is_some_punct(tt: Option<&TokenTree>, c: char) -> bool {
    if let Some(TokenTree::Punct(p)) = tt && p.as_char() == c {true} else {false}
  }
  fn expect_punct(tt: &TokenTree, c: char) -> Result<(), Error> {
    Self::is_punct(tt, c)
      .then_some(())
      .ok_or_else(|| Error::new_at_tokens(&tt, format!("Expected '{}' but found: '{}'", c, tt)))
  }
  fn take_path_sep(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<bool, Error> {
    if Self::is_some_punct(input.peek(), ':') {
      let last = input.next();
      let next = input.next();
      if let Some(next) = next {
        Self::expect_punct(&next, ':')?;
        Ok(true)
      } else {
        Err(Error::new_at_tokens(last, "Expected ':' but got to end"))
      }
    } else {
      Ok(false)
    }
  }
  fn as_ident(input: Option<TokenTree>) -> Result<Ident, Error> {
    input.map_or_else(
      || Err(Error::new("Expected Identifier")),
      |input| {
        if let TokenTree::Ident(i) = input {
          Ok(i)
        } else {
          Err(Error::new_at_tokens(
            &input,
            format!("Expected Identifier but found: '{}'", input),
          ))
        }
      },
    )
  }
  fn parse(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Self, Error> {
    let fully_qualified = Self::take_path_sep(input)?;

    let mut ty = Self {
      fully_qualified,
      path: vec![Self::as_ident(input.next())?],
      generics: vec![],
    };

    while let Some(tt) = input.peek() {
      let err_msg = Error::new_at_tokens(tt, format!("Expected ::, <, > or , but found: '{}'", tt));
      if Self::is_punct(tt, '<') {
        input.next();
        ty.generics = Self::parse_vec_generics(input)?;
        if let Some(tt) = input.peek() {
          if Self::is_punct(tt, '>') {
          } else if Self::is_punct(tt, ',') {
            input.next();
          } else {
            unreachable!("Got {} at unexpected place", tt) // i think, otherwise give some error
          }
        }
        break;
      } else if Self::is_punct(tt, '>') {
        break;
      } else if Self::is_punct(tt, ',') {
        input.next();
        break;
      } else if Self::take_path_sep(input)? {
        ty.path.push(Self::as_ident(input.next())?);
      } else {
        return Err(err_msg);
      }
    }
    Ok(ty)
  }
  fn parse_vec(mut input: Peekable<impl Iterator<Item = TokenTree>>) -> Result<Vec<Self>, Error> {
    let ret = Self::parse_vec_inner(&mut input)?;
    input.next().map_or(Ok(ret), |tt| {
      Err(Error::new_at_tokens(
        &tt,
        format!("Unconsumed Input '{}'", tt),
      ))
    })
  }

  fn parse_vec_generics(
    input: &mut Peekable<impl Iterator<Item = TokenTree>>,
  ) -> Result<Vec<Self>, Error> {
    let ret = Self::parse_vec_inner(input)?;
    if let Some(end) = input.next() {
      Self::expect_punct(&end, '>')?;
      Ok(ret)
    } else {
      Err(Error::new("Expected '>' but got to end"))
    }
  }
  fn parse_vec_inner(
    input: &mut Peekable<impl Iterator<Item = TokenTree>>,
  ) -> Result<Vec<Self>, Error> {
    let mut out = vec![];
    while let Some(tt) = input.peek() {
      if Self::is_punct(tt, '>') {
        return Ok(out);
      }
      out.push(Self::parse(input)?);
    }
    Ok(out)
  }
  fn span(&self) -> Span {
    self.path[self.path.len() - 1].span()
  }
  fn to_ident_string(&self) -> String {
    let mut res = String::new();
    if self.fully_qualified {
      res.push_str("_s_");
    }
    res.push_str(&self.path[0].to_string());
    for part in &self.path[1..] {
      res.push_str(&format!("_s_{}", part));
    }
    if !self.generics.is_empty() {
      res.push_str(&format!("_gs_{}", self.generics[0].to_ident_string()));
      for generic in &self.generics[1..] {
        res.push_str(&format!("_g_{}", generic.to_ident_string()));
      }
      res.push_str("_sg_");
    }
    res
  }
}
impl Display for Type {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    if self.fully_qualified {
      write!(f, "::")?;
    }
    write!(f, "{}", self.path[0])?;
    for part in &self.path[1..] {
      write!(f, "::{}", part)?;
    }
    if !self.generics.is_empty() {
      write!(f, "<{}", self.generics[0])?;
      for generic in &self.generics[1..] {
        write!(f, ", {}", generic)?;
      }
      write!(f, ">")?;
    }
    Ok(())
  }
}
impl ToTokens for Type {
  fn to_tokens(&self, stream: &mut TokenStream) {
    if self.fully_qualified {
      stream.append_all(quote!(::));
    }
    let first = &self.path[0];
    stream.append_all(quote!(#first));
    for part in &self.path[1..] {
      stream.append_all(quote!(::#part));
    }
    if !self.generics.is_empty() {
      stream.append_all(quote!(<));
      stream.append_separated(self.generics.iter(), quote!(,));
      stream.append_all(quote!(>));
    }
  }
}

fn gen_mapping_funcs(item_name: &Ident, args: &[Type]) -> TokenStream {
  let to_dyn_funcs = args
    .iter()
    .map(|ident| {
      let to_dyn_ref_name = format_ident!("__internal_to_dyn_ref_{}", ident.to_ident_string());
      let to_dyn_mut_name = format_ident!("__internal_to_dyn_mut_{}", ident.to_ident_string());
      #[cfg(feature = "downcast_unchecked")]
      let ret = quote_spanned!(ident.span() =>
        pub fn #to_dyn_ref_name(input: &dyn ::trait_cast_rs::Traitcastable) -> ::core::option::Option<&(dyn #ident + 'static)> {
          let casted: &Self = unsafe { input.downcast_ref_unchecked() };
          // SAFETY:
          //   This is safe since we know that `input` is a instance of Self.
          Some( casted as &dyn #ident)
        }
        pub fn #to_dyn_mut_name(input: &mut dyn ::trait_cast_rs::Traitcastable) -> ::core::option::Option<&mut (dyn #ident + 'static)> {
          let casted: &mut Self = unsafe { input.downcast_mut_unchecked() };
          // SAFETY:
          //   This is safe since we know that `input` is a instance of Self.
          Some( casted as &mut dyn #ident)
        }
      );
      #[cfg(not(feature = "downcast_unchecked"))]
      let ret = quote_spanned!(ident.span() =>
        pub fn #to_dyn_ref_name(input: &dyn ::trait_cast_rs::Traitcastable) -> ::core::option::Option<&(dyn #ident + 'static)> {
          let casted: Option<&Self> = input.downcast_ref();
          casted.map(|selv| selv as &dyn #ident)
        }
        pub fn #to_dyn_mut_name(input: &mut dyn ::trait_cast_rs::Traitcastable) -> ::core::option::Option<&mut (dyn #ident + 'static)> {
          let casted:  Option<&mut Self> = input.downcast_mut();
          casted.map(|selv| selv as &mut dyn #ident)
        }
      );
      ret
    })
    .collect::<TokenStream>();
  let expanded = quote!(
    impl #item_name {
      #to_dyn_funcs
    }
  );
  expanded
}

fn gen_target_func(item_name: &Ident, args: &[Type]) -> TokenStream {
  let targets = args
    .iter()
    .map(|ident| {
      let to_dyn_ref_name = format_ident!("__internal_to_dyn_ref_{}", ident.to_ident_string());
      let to_dyn_mut_name = format_ident!("__internal_to_dyn_mut_{}", ident.to_ident_string());
      quote_spanned!(ident.span() =>
        ::trait_cast_rs::TraitcastTarget::create(#item_name::#to_dyn_ref_name, #item_name::#to_dyn_mut_name),
      )
    })
    .collect::<TokenStream>();
  let expanded = quote!(
    impl ::trait_cast_rs::Traitcastable for #item_name {
      fn traitcast_targets(&self) -> &'static [::trait_cast_rs::TraitcastTarget] {
        const TARGETS: &'static [::trait_cast_rs::TraitcastTarget] = &[ #targets ];
        TARGETS
      }
    }
  );
  expanded
}
/// Attribute macro implementing `Traitcastable` for a struct, enum or union.
///
/// Use the arguments to specify all possible target Traits for witch trait objects are
/// supposed to be downcastable from a dyn `Traitcastable`.
///
/// Example:
/// ```no_build
///   extern crate trait_cast_rs;
///
///   use trait_cast_rs::{make_trait_castable, TraitcastTarget, TraitcastTo, Traitcastable};
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
///     let castable: Box<dyn Traitcastable> = source;
///     let x: &dyn Print = castable.downcast_ref().unwrap();
///     x.print();
///   }
/// ```
#[proc_macro_attribute]
pub fn make_trait_castable(args: TokenStream1, input: TokenStream1) -> TokenStream1 {
  let args = match Type::parse_vec(TokenStream::from(args).into_iter().peekable()) {
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
  TokenStream1::from(quote!(
    #input
    #mapping_funcs
    #target_func
  ))
}
