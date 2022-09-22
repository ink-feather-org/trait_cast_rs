#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![feature(trait_upcasting)]
#![allow(incomplete_features)]

use trait_cast_rs::{
  TraitcastTarget, TraitcastableAny, TraitcastableAnyInfra, TraitcastableAnyInfraExt,
  TraitcastableTo,
};

struct HybridPet {
  name: String,
}
impl TraitcastableTo<dyn Dog> for HybridPet {
  fn to_dyn_ref(input: &dyn TraitcastableAny) -> Option<&(dyn Dog + 'static)> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Dog)
  }

  fn to_dyn_mut(input: &mut dyn TraitcastableAny) -> Option<&mut (dyn Dog + 'static)> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Dog)
  }
}

impl TraitcastableTo<dyn Cat> for HybridPet {
  fn to_dyn_ref(input: &dyn TraitcastableAny) -> Option<&dyn Cat> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Cat)
  }

  fn to_dyn_mut(input: &mut dyn TraitcastableAny) -> Option<&mut dyn Cat> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Cat)
  }
}

impl TraitcastableAny for HybridPet {
  fn traitcast_targets(&self) -> &[TraitcastTarget] {
    const TARGETS: &'static [TraitcastTarget] = &[
      TraitcastTarget::from::<HybridPet, dyn Dog>(),
      TraitcastTarget::from::<HybridPet, dyn Cat>(),
    ];
    TARGETS
  }
}
impl HybridPet {
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
