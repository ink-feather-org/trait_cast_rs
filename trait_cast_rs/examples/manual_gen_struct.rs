#![cfg_attr(feature = "min_specialization", feature(min_specialization))]

use std::{any::type_name, fmt::Display};

use trait_cast_rs::{TraitcastTarget, TraitcastableAny, TraitcastableAnyInfra, TraitcastableTo};

struct HybridPet<T: Display> {
  name: T,
}
impl<T: Display> HybridPet<T> {
  fn greet(&self) {
    println!("{}: Hi {}", self.name, type_name::<T>())
  }
}

impl<T: Display> Dog for HybridPet<T> {
  fn bark(&self) {
    println!("{}: Woof!", self.name);
  }
}
impl<V: Display, T: Display> Cat<V> for HybridPet<T> {
  fn meow(&self, speak: &V) {
    println!("{}: Meow! {}", self.name, speak);
  }
}

trait Dog {
  fn bark(&self);
}

trait Cat<T: Display> {
  fn meow(&self, speak: &T);
}

impl<T: Display + 'static> TraitcastableTo<dyn Dog> for HybridPet<T> {
  fn to_dyn_ref(input: &dyn TraitcastableAny) -> Option<&(dyn Dog + 'static)> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Dog)
  }

  fn to_dyn_mut(input: &mut dyn TraitcastableAny) -> Option<&mut (dyn Dog + 'static)> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Dog)
  }
}

impl<T: Display + 'static, V: Display + 'static> TraitcastableTo<dyn Cat<V>> for HybridPet<T> {
  fn to_dyn_ref(input: &dyn TraitcastableAny) -> Option<&(dyn Cat<V> + 'static)> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Cat<V>)
  }

  fn to_dyn_mut(input: &mut dyn TraitcastableAny) -> Option<&mut (dyn Cat<V> + 'static)> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Cat<V>)
  }
}

// Next line is no no because of "use of generic parameter from outer function"
// would have to fall back to lazy_static
//impl<T: Display + 'static> TraitcastableAny for HybridPet<T> {
impl<T: Display + 'static> TraitcastableAny for HybridPet<T> {
  fn traitcast_targets(&self) -> &[TraitcastTarget] {
    const TARGETS: &[TraitcastTarget] = &[
      TraitcastTarget::new(
        HybridPet::<T>::to_dyn_ref_dog,
        HybridPet::<T>::to_dyn_mut_dog,
      ),
      TraitcastTarget::new(
        HybridPet::<T>::to_dyn_ref_cat::<String>,
        HybridPet::<T>::to_dyn_mut_cat::<String>,
      ),
    ];
    TARGETS
  }
}

fn main() {
  // The box is technically not needed but kept for added realism
  // let pet = Box::new(HybridPet {
  //   name: "Kokusnuss".to_string(),
  // });
  // pet.greet();

  // let castable_pet: Box<dyn TraitcastableAny> = pet;

  // let as_dog: &dyn Dog = castable_pet.downcast_ref().unwrap();
  // as_dog.bark();

  // let as_cat: &dyn Cat = castable_pet.downcast_ref().unwrap();
  // as_cat.meow();

  // let cast_back: &HybridPet = castable_pet.downcast_ref().unwrap();
  // cast_back.greet();

  // let no_mouse = <dyn TraitcastableAny as TraitcastTo<dyn Mouse>>::downcast(castable_pet);
  // if let Err(no_mouse) = no_mouse {
  //   let as_cat: &dyn Cat = no_mouse.downcast_ref().unwrap();
  //   as_cat.meow();
  // }
}
