#![feature(trait_upcasting, const_type_id)]
#![allow(incomplete_features)]
use std::any::Any;

use trait_cast_rs::{trait_cast, TraitcastTarget, Traitcastable};

extern crate trait_cast_rs;

struct HybridPet {
  name: String,
}
impl HybridPet {
  /// Pass this function pointer to register_downcast
  pub fn to_dyn_dog(input: &dyn Traitcastable) -> Option<&dyn Dog> {
    let any: &dyn Any = input;
    any.downcast_ref::<Self>().map(|selv| selv as &dyn Dog)
  }
  /// Pass this function pointer to register_downcast
  pub fn to_dyn_cat(input: &dyn Traitcastable) -> Option<&dyn Cat> {
    let any: &dyn Any = input;
    any.downcast_ref::<Self>().map(|selv| selv as &dyn Cat)
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
impl Traitcastable for HybridPet {
  fn traitcastable_from(&self) -> &'static [TraitcastTarget] {
    const TARGETS: &'static [TraitcastTarget] = unsafe {
      &[
        TraitcastTarget::new(
          std::any::TypeId::of::<dyn Dog>(),
          std::mem::transmute(HybridPet::to_dyn_dog as fn(_) -> _),
        ),
        TraitcastTarget::new(
          std::any::TypeId::of::<dyn Cat>(),
          std::mem::transmute(HybridPet::to_dyn_cat as fn(_) -> _),
        ),
      ]
    };
    TARGETS
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
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });

  let any_pet: Box<dyn Traitcastable> = pet;

  // WARNING: YOU MUST USE `as_ref()` otherwise you would cast the Box to an Any!
  let as_dog = trait_cast::<dyn Dog>(any_pet.as_ref()).unwrap();
  as_dog.bark();

  let as_cat = trait_cast::<dyn Cat>(any_pet.as_ref()).unwrap();
  as_cat.meow();
}
