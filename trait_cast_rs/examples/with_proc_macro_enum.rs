#![feature(trait_upcasting, const_type_id)]
#![allow(incomplete_features)]
use std::any::Any;

use trait_cast_rs::{make_trait_castable, TraitcastTarget, Traitcastable};

extern crate trait_cast_rs;

#[make_trait_castable(Dog, Cat)]
enum HybridPet {
  Name(String),
}
impl HybridPet {
  fn greet(&self) {
    let Self::Name(name) = self;
    println!("{}: Hi", name)
  }
}

impl Dog for HybridPet {
  fn bark(&self) {
    let Self::Name(name) = self;
    println!("{}: Woof!", name)
  }
}
impl Cat for HybridPet {
  fn meow(&self) {
    let Self::Name(name) = self;
    println!("{}: Meow!", name)
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

  let as_dog = castable_pet.trait_cast_ref::<dyn Dog>().unwrap();
  as_dog.bark();

  let as_cat = castable_pet.trait_cast_ref::<dyn Cat>().unwrap();
  as_cat.meow();

  let cast_back = castable_pet.downcast_ref::<HybridPet>().unwrap();
  cast_back.greet();
}
