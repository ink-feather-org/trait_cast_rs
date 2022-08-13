#![no_std]
#![deny(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]
#![allow(incomplete_features)]
#![feature(
  const_type_id,
  const_type_name,
  trait_upcasting,
  const_mut_refs,
  min_specialization,
  doc_cfg
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
