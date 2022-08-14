use trait_cast_rs::{make_trait_castable, TraitcastTo, Traitcastable};

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

  let as_dog: &mut dyn Dog = castable_pet.downcast_mut().unwrap();
  as_dog.rename("Rommel".to_string());

  let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  cast_back.greet();
}
