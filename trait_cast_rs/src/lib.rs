#![no_std]
#![allow(incomplete_features)]
#![feature(const_type_id, trait_upcasting)]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod trait_cast;
pub use trait_cast::*;

pub use trait_cast_impl_rs::make_trait_castable;

#[cfg(test)]
mod test;
