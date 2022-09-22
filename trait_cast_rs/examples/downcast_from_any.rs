#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]
#![cfg_attr(feature = "const_sort", feature(const_trait_impl, const_mut_refs))]
#![feature(trait_upcasting)]
#![allow(incomplete_features)]

#[cfg(feature = "min_specialization")]
mod min_specialization {

  use std::any::Any;

  use trait_cast_rs::{make_trait_castable, DowncastFromAny, TraitcastableAny};

  #[make_trait_castable()]
  struct Woof;

  pub(crate) fn main() {
    let castable_pet: &dyn TraitcastableAny = &Woof;

    let as_any: &dyn Any = castable_pet;

    let as_caster: &dyn DowncastFromAny = as_any;

    let _as_castable_pet: &dyn TraitcastableAny = as_caster;
  }
}

#[cfg_attr(test, test)]
#[cfg(feature = "min_specialization")]
fn main() {
  min_specialization::main()
}

#[cfg(not(feature = "min_specialization"))]
const fn main() {}
