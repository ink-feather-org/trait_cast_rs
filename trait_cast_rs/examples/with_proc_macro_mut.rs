#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]
#![cfg_attr(feature = "const_sort", feature(const_trait_impl, const_mut_refs))]

use trait_cast_rs::{make_trait_castable, TraitcastableAny, TraitcastableAnyInfra};

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
    println!("Changing name from \"{}\" to \"{new_name}\"", self.name);
    self.name = new_name;
  }
}

trait Dog {
  fn rename(&mut self, new_tag: String);
}
#[cfg_attr(test, test)]
fn main() {
  // The box is technically not needed but kept for added realism
  let pet = Box::new(HybridPet {
    name: "Kokusnuss".to_string(),
  });
  pet.greet();

  let mut castable_pet: Box<dyn TraitcastableAny> = pet;

  let as_dog: &mut dyn Dog = castable_pet.downcast_mut().unwrap();
  as_dog.rename("Rommel".to_string());

  let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  cast_back.greet();
}
