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
impl<V: Display + ?Sized, T: Display> Cat<V> for HybridPet<T> {
  fn meow(&self, speak: &V) {
    println!("{}: Meow! {}", self.name, speak);
  }
}

trait Dog {
  fn bark(&self);
}

trait Cat<T: Display + ?Sized> {
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

impl<T: Display + 'static, V: Display + 'static + ?Sized> TraitcastableTo<dyn Cat<V>>
  for HybridPet<T>
{
  fn to_dyn_ref(input: &dyn TraitcastableAny) -> Option<&(dyn Cat<V> + 'static)> {
    let casted: Option<&Self> = input.downcast_ref();
    casted.map(|selv| selv as &dyn Cat<V>)
  }

  fn to_dyn_mut(input: &mut dyn TraitcastableAny) -> Option<&mut (dyn Cat<V> + 'static)> {
    let casted: Option<&mut Self> = input.downcast_mut();
    casted.map(|selv| selv as &mut dyn Cat<V>)
  }
}

// The `TARGETS` slice can not be declared inside the `traitcast_targets` function.
// The "use of generic parameter from outer function" rust limitation is the cause.
impl<T: Display + 'static> HybridPet<T> {
  const TARGETS: &[TraitcastTarget] = &[
    TraitcastTarget::from::<Self, dyn Dog>(),
    TraitcastTarget::from::<Self, dyn Cat<str>>(),
  ];
}

impl<T: Display + 'static> TraitcastableAny for HybridPet<T> {
  fn traitcast_targets(&self) -> &[TraitcastTarget] {
    Self::TARGETS
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

  let as_cat: &dyn Cat<str> = castable_pet.downcast_ref().unwrap();
  as_cat.meow("Text");

  let cast_back: &HybridPet<String> = castable_pet.downcast_ref().unwrap();
  cast_back.greet();

  // Concrete generic `Cat<String>` not specified as a target for `HybridPet<String>`.
  // Adding `TraitcastTarget::from::<Self, dyn Cat<String>>(),` to the targets would make the cast valid.
  let invalid_cast: Option<&dyn Cat<String>> = castable_pet.downcast_ref();
  assert!(invalid_cast.is_none());
}
