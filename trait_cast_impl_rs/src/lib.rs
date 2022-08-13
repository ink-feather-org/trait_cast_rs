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
  fn expect_punct(tt: TokenTree, c: char) -> Result<(), Error> {
    Self::is_punct(&tt, c)
      .then_some(())
      .ok_or_else(|| Error::new_at_tokens(&tt, format!("Expected '{}' but found: '{}'", c, tt)))
  }
  fn take_path_sep(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<bool, Error> {
    if Self::is_some_punct(input.peek(), ':') {
      let last = input.next();
      let next = input.next();
      if let Some(next) = next {
        Self::expect_punct(next, ':')?;
        Ok(true)
      } else {
        Err(Error::new_at_tokens(last, "Expected ':' but got to end"))
      }
    } else {
      Ok(false)
    }
  }
  fn as_ident(input: Option<TokenTree>) -> Result<Ident, Error> {
    if let Some(input) = input {
      if let TokenTree::Ident(i) = input {
        Ok(i)
      } else {
        Err(Error::new_at_tokens(
          &input,
          format!("Expected Identifier but found: '{}'", input),
        ))
      }
    } else {
      Err(Error::new("Expected Identifier"))
    }
  }
  fn parse(mut input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Type, Error> {
    //let mut input = input.into_iter().peekable();
    let fully_qualified = Self::take_path_sep(&mut input)?;

    let mut ty = Type {
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
            todo!("Err2");
          }
        }
        break;
      } else if Self::is_punct(tt, '>') {
        break;
      } else if Self::is_punct(tt, ',') {
        input.next();
        break;
      } else if Self::take_path_sep(&mut input)? {
        ty.path.push(Self::as_ident(input.next())?);
      } else {
        return Err(err_msg);
      }
    }
    Ok(ty)
  }
  fn parse_vec(mut input: Peekable<impl Iterator<Item = TokenTree>>) -> Result<Vec<Type>, Error> {
    let ret = Self::parse_vec_inner(&mut input)?;
    if let Some(tt) = input.next() {
      Err(Error::new_at_tokens(
        &tt,
        format!("Unconsumed Input '{}'", tt),
      ))
    } else {
      Ok(ret)
    }
  }

  fn parse_vec_generics(
    mut input: &mut Peekable<impl Iterator<Item = TokenTree>>,
  ) -> Result<Vec<Type>, Error> {
    let ret = Self::parse_vec_inner(&mut input)?;
    if let Some(end) = input.next() {
      Self::expect_punct(end, '>')?;
      //dbg!(input.peek());
      Ok(ret)
    } else {
      Err(Error::new("Expected '>' but got to end"))
    }
  }
  fn parse_vec_inner(
    input: &mut Peekable<impl Iterator<Item = TokenTree>>,
  ) -> Result<Vec<Type>, Error> {
    let mut out = vec![];
    while let Some(tt) = input.peek() {
      if Self::is_punct(tt, '>') {
        return Ok(out);
      }
      out.push(Self::parse(input)?)
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
      stream.append_all(quote!(::))
    }
    let first = &self.path[0];
    stream.append_all(quote!(#first));
    for part in &self.path[1..] {
      stream.append_all(quote!(::#part))
    }
    if !self.generics.is_empty() {
      stream.append_all(quote!(<));
      stream.append_separated(self.generics.iter(), quote!(,));
      stream.append_all(quote!(>));

      //todo!("generic_to_token")
    }
    //stream.append_all(quote!(Dog))
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
        pub fn #to_dyn_ref_name(input: &dyn Traitcastable) -> Option<&(dyn #ident + 'static)> {
          let any: &dyn Any = input;
          Some( unsafe {any.downcast_ref_unchecked::<Self>() as &dyn #ident})
        }
        pub fn #to_dyn_mut_name(input: &mut dyn Traitcastable) -> Option<&mut (dyn #ident + 'static)> {
          let any: &mut dyn Any = input;
          Some( unsafe {any.downcast_mut_unchecked::<Self>() as &mut dyn #ident})
        }
      );
      #[cfg(not(feature = "downcast_unchecked"))]
      let ret = quote_spanned!(ident.span() =>
        pub fn #to_dyn_ref_name(input: &dyn Traitcastable) -> Option<&(dyn #ident + 'static)> {
          let any: &dyn Any = input;
          any.downcast_ref::<Self>().map(|selv| selv as &dyn #ident)
        }
        pub fn #to_dyn_mut_name(input: &mut dyn Traitcastable) -> Option<&mut (dyn #ident + 'static)> {
          let any: &mut dyn Any = input;
          any.downcast_mut::<Self>().map(|selv| selv as &mut dyn #ident)
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
        TraitcastTarget::create(#item_name::#to_dyn_ref_name, #item_name::#to_dyn_mut_name),
      )
    })
    .collect::<TokenStream>();
  let expanded = quote!(
    impl ::trait_cast_rs::Traitcastable for #item_name {
      fn traitcast_targets(&self) -> &'static [TraitcastTarget] {
        const TARGETS: &'static [TraitcastTarget] = &[ #targets ];
        TARGETS
      }
    }
  );
  expanded
}

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
