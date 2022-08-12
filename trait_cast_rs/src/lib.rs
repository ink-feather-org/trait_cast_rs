#![no_std]
mod trait_cast;
pub use trait_cast::*;

pub use trait_cast_impl_rs::make_trait_castable;

#[cfg(test)]
mod test;
