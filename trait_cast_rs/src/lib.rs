#![doc = include_str!("../README.md")]
#![no_std]
#![expect(
  unsafe_code,
  reason = "The function transmutations require unsafe code."
)]
#![allow(incomplete_features)]
#![feature(
  const_type_id,      // Needed to enable `TraitcastTarget::create` to be const
  const_type_name,    // Needed for `Debug` implementation
  trait_upcasting,    // Needed to avoid reimplementing Any
  min_specialization, // Needed to unify the interface between downcast and traitcast (could be avoided with !Trait bounds or trait generics)
  ptr_metadata,       // Needed to deal with pointer address(and provenance) separately from metadata
  doc_cfg             // For nicer Docs
)]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod trait_cast;
pub use trait_cast::*;

mod decl_macro;

pub use trait_cast_impl_rs::make_trait_castable;

#[cfg(test)]
mod test;
