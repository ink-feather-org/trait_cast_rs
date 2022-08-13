#![feature(trait_upcasting, const_type_id)]
#![allow(incomplete_features)]
use std::any::{type_name, Any};

use trait_cast_rs::{make_trait_castable, trait_cast, TraitcastTarget, Traitcastable};

extern crate trait_cast_rs;

#[make_trait_castable(Dog<i32>, Dog<TestStruct<::std::primitive::i32>>, Cat)]
struct HybridPet {
  name: String,
}
struct TestStruct<T>(T);
impl HybridPet {
  fn greet(&self) {
    println!("{}: Hi", self.name)
  }
}

impl<T> Dog<T> for HybridPet {
  fn bark(&self) {
    println!("{}: Woof({})!", self.name, type_name::<T>());
  }
}
impl Cat for HybridPet {
  fn meow(&self) {
    println!("{}: Meow!", self.name);
  }
}
trait Dog<T> {
  fn bark(&self);
}
trait Cat {
  fn meow(&self);
}

fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });
  pet.greet();

  let castable_pet: Box<dyn Traitcastable> = pet;

  let as_dog = trait_cast::<dyn Dog<i32>>(castable_pet.as_ref()).unwrap();
  as_dog.bark();

  let as_dog = trait_cast::<dyn Dog<TestStruct<i32>>>(castable_pet.as_ref()).unwrap();
  as_dog.bark();

  let as_cat = trait_cast::<dyn Cat>(castable_pet.as_ref()).unwrap();
  as_cat.meow();

  let any_pet = castable_pet as Box<dyn Any>;
  let cast_back: &HybridPet = any_pet.downcast_ref().unwrap();
  cast_back.greet();
}
