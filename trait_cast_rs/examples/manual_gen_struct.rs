#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![feature(ptr_metadata)]

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
    println!("{}: Meow! {speak}", self.name);
  }
}

trait Dog {
  fn bark(&self);
}

trait Cat<T: Display + ?Sized> {
  fn meow(&self, speak: &T);
}

impl<T: Display + 'static> TraitcastableTo<dyn Dog> for HybridPet<T> {
  const METADATA: ::core::ptr::DynMetadata<dyn Dog> = {
    let ptr: *const HybridPet<T> = ::core::ptr::from_raw_parts(::core::ptr::null(), ());
    let ptr: *const dyn Dog = ptr as _;

    ptr.to_raw_parts().1
  };
}

impl<T: Display + 'static, V: Display + 'static + ?Sized> TraitcastableTo<dyn Cat<V>>
  for HybridPet<T>
{
  const METADATA: ::core::ptr::DynMetadata<dyn Cat<V>> = {
    let ptr: *const HybridPet<T> = ::core::ptr::from_raw_parts(::core::ptr::null(), ());
    let ptr: *const dyn Cat<V> = ptr as _;

    ptr.to_raw_parts().1
  };
}

// The `TARGETS` slice can not be declared inside the `traitcast_targets` function.
// The "use of generic parameter from outer function" rust limitation is the cause.
impl<T: Display + 'static> HybridPet<T> {
  const TARGETS: &[TraitcastTarget] = &[
    TraitcastTarget::from::<Self, dyn Dog>(),
    TraitcastTarget::from::<Self, dyn Cat<str>>(),
  ];
}

unsafe impl<T: Display + 'static> TraitcastableAny for HybridPet<T> {
  fn traitcast_targets(&self) -> &[TraitcastTarget] {
    Self::TARGETS
  }
}

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

  let as_cat: &dyn Cat<str> = castable_pet.downcast_ref().unwrap();
  as_cat.meow("Text");

  let cast_back: &HybridPet<String> = castable_pet.downcast_ref().unwrap();
  cast_back.greet();

  // Concrete generic `Cat<String>` not specified as a target for `HybridPet<String>`.
  // Adding `TraitcastTarget::from::<Self, dyn Cat<String>>(),` to the targets would make the cast valid.
  let invalid_cast: Option<&dyn Cat<String>> = castable_pet.downcast_ref();
  assert!(invalid_cast.is_none());
}
