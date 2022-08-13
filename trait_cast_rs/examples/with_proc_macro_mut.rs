#![feature(trait_upcasting, const_type_id)]
#![allow(incomplete_features)]
use std::any::Any;

use trait_cast_rs::{make_trait_castable, TraitcastTarget, Traitcastable};

extern crate trait_cast_rs;

#[make_trait_castable(Dog)]
struct HybridPet {
  name: String,
}
impl HybridPet {
  fn greet(&self) {
    println!("{}: Hi", self.name)
  }
}

impl Dog for HybridPet {
  fn rename(&mut self, new_name: String) {
    println!("Changing name from \"{}\" to \"{}\"", self.name, new_name);
    self.name = new_name;
  }
}

trait Dog {
  fn rename(&mut self, new_tag: String);
}

fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });
  pet.greet();

  let mut castable_pet: Box<dyn Traitcastable> = pet;

  let as_dog = castable_pet.trait_cast_mut::<dyn Dog>().unwrap();
  as_dog.rename("Rommel".to_string());

  let cast_back = castable_pet.downcast_ref::<HybridPet>().unwrap();
  cast_back.greet();
}
