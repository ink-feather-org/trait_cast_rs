#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]
#![feature(trait_upcasting)]
#![allow(incomplete_features)]

use std::any::Any;

use trait_cast_rs::{make_trait_castable_decl, TraitcastableAny, TraitcastableAnyInfra};

extern crate trait_cast_rs;

struct HybridPet {
  name: String,
}

struct HybridAnimal {
  name: String,
}

make_trait_castable_decl! {
    HybridPet => (Dog, Cat),
    HybridAnimal => (Cow, Cat),
}

impl HybridPet {
  fn greet(&self) {
    println!("{}: Hi", self.name)
  }
}

impl HybridAnimal {
  fn greet(&self) {
    println!("{}: Hi", self.name)
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

impl Cow for HybridAnimal {
  fn moo(&self) {
    println!("{}: Moo!", self.name);
  }
}
impl Cat for HybridAnimal {
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
trait Cow {
  fn moo(&self);
}

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

  let any_pet = castable_pet as Box<dyn Any>;
  let cast_back: &HybridPet = any_pet.downcast_ref().unwrap();
  cast_back.greet();

  // The box is technically not needed but kept for added realism
  let animal = Box::new(HybridAnimal {
    name: "Cyow".to_string(),
  });
  animal.greet();

  let castable_animal: Box<dyn TraitcastableAny> = animal;

  let as_cat: &dyn Cat = castable_animal.downcast_ref().unwrap();
  as_cat.meow();

  let as_cow: &dyn Cow = castable_animal.downcast_ref().unwrap();
  as_cow.moo();

  let any_animal = castable_animal as Box<dyn Any>;
  let cast_back: &HybridAnimal = any_animal.downcast_ref().unwrap();
  cast_back.greet();
}
