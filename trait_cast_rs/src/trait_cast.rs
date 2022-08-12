use core::{
  any::{Any, TypeId},
  mem,
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub struct TraitcastTarget {
  target_type_id: TypeId,
  to_dyn_func: fn(&dyn Traitcastable) -> *const (),
}
impl TraitcastTarget {
  pub const fn create<Target: 'static + ?Sized>(
    to_dyn_func: fn(&dyn Traitcastable) -> Option<&Target>,
  ) -> Self {
    Self {
      target_type_id: TypeId::of::<Target>(),
      to_dyn_func: unsafe { mem::transmute(to_dyn_func) },
    }
  }
}

pub trait Traitcastable: Any {
  fn traitcast_targets(&self) -> &'static [TraitcastTarget];

  fn type_id(&self) -> TypeId {
    Any::type_id(self)
  }
}

macro_rules! implement_any_features {
  (Any $(+ $traits:ident)*) => {
    impl dyn Traitcastable + $($traits +)* {
      pub fn is<T: Any $(+ $traits)*>(&self) -> bool {
        <dyn Any>::is::<T>(self)
      }
      pub fn downcast_ref<T: Any $(+ $traits)*>(&self) -> Option<&T> {
        <dyn Any>::downcast_ref::<T>(self)
      }
      pub fn downcast_mut<T: Any $(+ $traits)*>(&mut self) -> Option<&mut T> {
        <dyn Any>::downcast_mut::<T>(self)
      }
      #[cfg(feature = "downcast_unchecked")]
      pub unsafe fn downcast_ref_unchecked<T: Any $(+ $traits)*>(&self) -> &T {
        <dyn Any>::downcast_ref_unchecked::<T>(self)
      }
      #[cfg(feature = "downcast_unchecked")]
      pub unsafe fn downcast_mut_unchecked<T: Any $(+ $traits)*>(&mut self) -> &mut T {
        <dyn Any>::downcast_mut_unchecked::<T>(self)
      }
    }
    #[cfg(feature = "alloc")]
    impl Box<dyn Traitcastable> {
      pub fn downcast<T: Any $(+ $traits)*>(self) -> Result<Box<T>, Self> {
        <Box<dyn Any>>::downcast(self)
      }
      pub unsafe fn downcast_unchecked<T: Any $(+ $traits)*>(self) -> Box<T> {
        <Box<dyn Any>>::downcast_unchecked(self)
      }
    }
  };
}

implement_any_features!(Any);
implement_any_features!(Any + Send);
implement_any_features!(Any + Send + Sync);
// Maybe support this once min_specialization is supported.
// pub fn trait_cast<'a, Target: Sized + 'static>(
//   source: &'a dyn Any,
//   trait_cast_target: &[TraitcastTarget],
// ) -> Option<&'a Target> {
//   source.downcast_ref::<Target>()
// }

pub fn trait_cast<Target: ?Sized + 'static>(source: &dyn Traitcastable) -> Option<&Target> {
  let target = source
    .traitcast_targets()
    .iter()
    .find(|possible| possible.target_type_id == TypeId::of::<Target>());

  target.and_then(|target| {
    let fn_ptr: fn(&dyn Traitcastable) -> Option<&Target> =
      unsafe { mem::transmute(target.to_dyn_func) };
    fn_ptr(source)
  })
}
