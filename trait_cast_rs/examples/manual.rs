//! This example demonstrates how to manually implement the `TraitcastableAny` trait for a non generic struct `HybridPet`.
#![allow(clippy::undocumented_unsafe_blocks, clippy::use_self)]
#![expect(
  unsafe_code,
  reason = "Manual traitcast implementations require unsafe code."
)]
#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![feature(trait_upcasting)]
#![allow(incomplete_features)]
#![feature(ptr_metadata)]

use trait_cast_rs::{
  TraitcastTarget, TraitcastableAny, TraitcastableAnyInfra, TraitcastableAnyInfraExt,
  TraitcastableTo,
};

struct HybridPet {
  name: String,
}
impl TraitcastableTo<dyn Dog> for HybridPet {
  const METADATA: ::core::ptr::DynMetadata<dyn Dog> = {
    let self_ptr: *const HybridPet = ::core::ptr::null::<HybridPet>();
    let dyn_ptr: *const dyn Dog = self_ptr as _;

    dyn_ptr.to_raw_parts().1
  };
}

impl TraitcastableTo<dyn Cat> for HybridPet {
  const METADATA: ::core::ptr::DynMetadata<dyn Cat> = {
    let self_ptr: *const Self = ::core::ptr::null::<Self>();
    let dyn_ptr: *const dyn Cat = self_ptr as _;

    dyn_ptr.to_raw_parts().1
  };
}

unsafe impl TraitcastableAny for HybridPet {
  fn traitcast_targets(&self) -> &[TraitcastTarget] {
    const TARGETS: &[TraitcastTarget] = &[
      TraitcastTarget::from::<HybridPet, dyn Dog>(),
      TraitcastTarget::from::<HybridPet, dyn Cat>(),
    ];
    TARGETS
  }
}
impl HybridPet {
  fn greet(&self) {
    println!("{}: Hi", self.name);
  }
}

impl Dog for HybridPet {
  fn bark(&self) {
    println!("{}: Woof!", self.name);
  }
}
impl Cat for HybridPet {
  fn meow(&self) {
    println!("{}: Meow!", self.name);
  }
}

trait Dog {
  fn bark(&self);
}
trait Cat: TraitcastableAny {
  fn meow(&self);
}
trait Mouse {}

#[cfg_attr(test, test)]
fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });
  pet.greet();

  let castable_pet: Box<dyn TraitcastableAny> = pet;

  let as_dog: &dyn Dog = castable_pet.downcast_ref().unwrap();
  as_dog.bark();

  let as_cat: &dyn Cat = castable_pet.downcast_ref().unwrap();
  as_cat.meow();

  let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  cast_back.greet();

  // upcasting examples
  // require feature flag trait_upcasting
  // you must also add TraitcastableAny to your trait
  let upcast_ref: &dyn TraitcastableAny = as_cat;
  let downcast_to_cat_again: &dyn Cat = upcast_ref.downcast_ref().unwrap();
  downcast_to_cat_again.meow();

  let as_box_cat: Box<dyn Cat> = castable_pet.downcast().unwrap();
  let castable_pet: Box<dyn TraitcastableAny> = as_box_cat;

  // failed cast example
  // shows how to recover the box without dropping it
  let no_mouse: Result<Box<dyn Mouse>, _> = castable_pet.downcast();
  if let Err(no_mouse) = no_mouse {
    let as_cat: &dyn Cat = no_mouse.downcast_ref().unwrap();
    as_cat.meow();
  }
}
