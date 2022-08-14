#![cfg_attr(feature = "min_specialization", feature(min_specialization))]

use trait_cast_rs::{TraitcastTarget, TraitcastTo, TraitcastableAny};

extern crate trait_cast_rs;

struct HybridPet {
  name: String,
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
trait Cat {
  fn meow(&self);
}
trait Mouse {}

impl HybridPet {
  /// Pass this function pointer to register_downcast
  pub fn to_dyn_ref_dog(input: &dyn TraitcastableAny) -> Option<&(dyn Dog + 'static)> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Dog)
  }
  pub fn to_dyn_mut_dog(input: &mut dyn TraitcastableAny) -> Option<&mut (dyn Dog + 'static)> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Dog)
  }
  pub fn to_dyn_ref_cat(input: &dyn TraitcastableAny) -> Option<&(dyn Cat + 'static)> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Cat)
  }
  pub fn to_dyn_mut_cat(input: &mut dyn TraitcastableAny) -> Option<&mut (dyn Cat + 'static)> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Cat)
  }
}
impl TraitcastableAny for HybridPet {
  fn traitcast_targets(&self) -> &'static [TraitcastTarget] {
    const TARGETS: &'static [TraitcastTarget] = &[
      TraitcastTarget::create(HybridPet::to_dyn_ref_dog, HybridPet::to_dyn_mut_dog),
      TraitcastTarget::create(HybridPet::to_dyn_ref_cat, HybridPet::to_dyn_mut_cat),
    ];
    TARGETS
  }
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

  let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  cast_back.greet();

  let no_mouse = <dyn TraitcastableAny as TraitcastTo<dyn Mouse>>::downcast(castable_pet);
  if let Err(no_mouse) = no_mouse {
    let as_cat: &dyn Cat = no_mouse.downcast_ref().unwrap();
    as_cat.meow();
  }
}
