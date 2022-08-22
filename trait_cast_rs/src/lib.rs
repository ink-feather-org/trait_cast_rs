#![no_std]
// TODO: #![deny(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]
#![allow(incomplete_features)]
#![feature(
  const_type_id,      // Needed to enable `TraitcastTarget::create` to be const
  const_type_name,    // Needed for `Debug` implementation
  trait_upcasting,    // Needed to avoid reimplementing Any
  const_mut_refs,     // Needed since arguments to `TraitcastTarget::create` need a function pointer with &mut argument and return type.
  min_specialization, // Needed to unify the interface between downcast and traitcast (could be avoided with !Trait bounds or trait generics)
  doc_cfg             // For nicer Docs
)]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]
//! TODO

#[cfg(feature = "alloc")]
extern crate alloc;

mod trait_cast;
pub use trait_cast::*;

pub use trait_cast_impl_rs::make_trait_castable;

#[cfg(test)]
mod test;
