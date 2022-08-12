#![feature(trait_upcasting, const_type_id)]
#![allow(incomplete_features)]
use std::any::Any;

use trait_cast_rs::{make_trait_castable, trait_cast, TraitcastTarget, Traitcastable};

extern crate trait_cast_rs;

#[make_trait_castable(Dog, Cat)]
enum HybridPet {
  Name(String),
}
impl HybridPet {
  fn greet(&self) {
    if let Self::Name(name) = self {
      println!("{}: Hi", name)
    }
  }
}

impl Dog for HybridPet {
  fn bark(&self) {
    if let Self::Name(name) = self {
      println!("{}: Woof!", name)
    }
  }
}
impl Cat for HybridPet {
  fn meow(&self) {
    if let Self::Name(name) = self {
      println!("{}: Meow!", name)
    }
  }
}

trait Dog {
  fn bark(&self);
}
trait Cat {
  fn meow(&self);
}

fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet::Name("Kokusnuss".to_string()));
  pet.greet();

  let castable_pet: Box<dyn Traitcastable> = pet;

  // WARNING: YOU MUST USE `as_ref()` otherwise you would cast the Box to an Any!
  let as_dog = trait_cast::<dyn Dog>(castable_pet.as_ref()).unwrap();
  as_dog.bark();

  let as_cat = trait_cast::<dyn Cat>(castable_pet.as_ref()).unwrap();
  as_cat.meow();

  let any_pet = castable_pet as Box<dyn Any>;
  let cast_back: &HybridPet = any_pet.downcast_ref().unwrap();
  cast_back.greet();
}
