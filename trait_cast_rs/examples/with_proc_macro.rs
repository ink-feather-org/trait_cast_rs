#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]

use trait_cast_rs::{
  make_trait_castable, TraitcastableAny, TraitcastableAnyInfra, TraitcastableAnyInfraExt,
};

#[make_trait_castable(Dog, Cat)]
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
  #[cfg(feature = "downcast_unchecked")]
  let cast_back: &HybridPet = unsafe { castable_pet.downcast_ref_unchecked() };
  #[cfg(not(feature = "downcast_unchecked"))]
  let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  cast_back.greet();

  let into_cat: Box<dyn Cat> = castable_pet.downcast().unwrap();
  into_cat.meow();
}
