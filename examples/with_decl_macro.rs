//! This example demonstrates how to use the `make_trait_castable_decl` macro to declare a type as traitcastable.
#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]
#![feature(trait_upcasting)]
#![allow(incomplete_features)]
#![feature(ptr_metadata)]

use core::any::Any;

use trait_cast::{TraitcastableAny, TraitcastableAnyInfra, make_trait_castable_decl};

struct HybridPet {
  name: String,
}

make_trait_castable_decl! {
  HybridPet => (Dog, Cat),
  // Multiple standalone entries in one macro invocation are possible.
  // HybridAnimal => (Cow, Cat),
  // Sunflower => (Flower, Plant)
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
trait Cat {
  fn meow(&self);
}
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

  let any_pet: Box<dyn Any> = castable_pet;
  let cast_back: &HybridPet = any_pet.downcast_ref().unwrap();
  cast_back.greet();
}
