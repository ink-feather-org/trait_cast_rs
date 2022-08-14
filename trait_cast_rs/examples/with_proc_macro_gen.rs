#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]

use std::any::type_name;

use trait_cast_rs::{make_trait_castable, TraitcastTo, Traitcastable};

extern crate trait_cast_rs;

#[make_trait_castable(Dog<i32>, Dog<TestStruct<::std::primitive::i32>>, Cat<u128, u32>)]
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
impl<B> Cat<u128, B> for HybridPet {
  fn meow(&self) {
    println!("{}: Meow(u128, {})!", self.name, type_name::<B>());
  }
}
trait Dog<T> {
  fn bark(&self);
}
trait Cat<A, B> {
  fn meow(&self);
}

fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });
  pet.greet();

  let castable_pet: Box<dyn Traitcastable> = pet;

  let as_dog: &dyn Dog<i32> = castable_pet.downcast_ref().unwrap();
  as_dog.bark();

  let as_dog: &dyn Dog<TestStruct<i32>> = castable_pet.downcast_ref().unwrap();
  as_dog.bark();

  let as_cat: &dyn Cat<u128, u32> = castable_pet.downcast_ref().unwrap();
  as_cat.meow();

  let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  cast_back.greet();
}
