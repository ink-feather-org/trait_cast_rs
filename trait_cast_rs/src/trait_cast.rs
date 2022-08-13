use core::{
  any::{type_name, Any, TypeId},
  fmt::{self, Debug, Formatter},
  mem,
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub struct TraitcastTarget {
  target_type_id: TypeId,
  target_type_name: &'static str,
  to_dyn_ref: fn(&dyn Traitcastable) -> *const (),
  to_dyn_mut: fn(&mut dyn Traitcastable) -> *mut (),
}
impl TraitcastTarget {
  pub const fn create<Target: 'static + ?Sized>(
    to_dyn_ref: fn(&dyn Traitcastable) -> Option<&Target>,
    to_dyn_mut: fn(&mut dyn Traitcastable) -> Option<&mut Target>,
  ) -> Self {
    Self {
      target_type_id: TypeId::of::<Target>(),
      target_type_name: type_name::<Target>(),
      to_dyn_ref: unsafe { mem::transmute(to_dyn_ref) },
      to_dyn_mut: unsafe { mem::transmute(to_dyn_mut) },
    }
  }
}

pub trait Traitcastable: Any {
  fn traitcast_targets(&self) -> &'static [TraitcastTarget];

  fn type_id(&self) -> TypeId {
    Any::type_id(self)
  }
}

macro_rules! implement_with_markers {
  ($($(+)? $traits:ident)*) => {
    impl Debug for dyn Traitcastable + $($traits +)* {
      fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Traitcastable to {{")?;
        for (i, target) in self.traitcast_targets().iter().enumerate() {
          if i != 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}", target.target_type_name)?;
        }
        write!(f, "}}")
      }
    }
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

      #[cfg(feature = "alloc")]
      pub fn downcast<T: Any $(+ $traits)*>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() { unsafe { Ok(self.downcast_unchecked::<T>()) } } else { Err(self) }
      }
      #[cfg(feature = "alloc")]
      pub unsafe fn downcast_unchecked<T: Any $(+ $traits)*>(self: Box<Self>) -> Box<T> {
        <Box<dyn Any>>::downcast_unchecked(self)
      }
    }
    impl dyn Traitcastable + $($traits +)* {
      pub fn trait_cast_ref<Target: ?Sized + 'static + $($traits +)*>(&self) -> Option<&Target> {
        let target = self
          .traitcast_targets()
          .iter()
          .find(|possible| possible.target_type_id == TypeId::of::<Target>());

        target.and_then(|target| {
          let fn_ptr: fn(&dyn Traitcastable) -> Option<&Target> =
            unsafe { mem::transmute(target.to_dyn_ref) };
          fn_ptr(self)
        })
      }

      pub fn trait_cast_mut<Target: ?Sized + 'static + $($traits +)*>(&mut self) -> Option<&mut Target> {
        let target = self
          .traitcast_targets()
          .iter()
          .find(|possible| possible.target_type_id == TypeId::of::<Target>());

        target.and_then(|target| {
          let fn_ptr: fn(&mut dyn Traitcastable) -> Option<&mut Target> =
            unsafe { mem::transmute(target.to_dyn_mut) };
          fn_ptr(self)
        })
      }

      #[cfg(feature = "alloc")]
      pub fn trait_cast<Target: ?Sized + 'static + $($traits +)*>(self: Box<Self>) -> Option<Box<Target>> {
        let raw: &mut Self = unsafe { &mut *Box::into_raw(self) };
        let to_ref: *mut Target = &mut *raw.trait_cast_mut::<Target>()?;
        Some(unsafe { Box::from_raw(to_ref) })
      }
    }
  };
}

implement_with_markers!();
implement_with_markers!(Send);
implement_with_markers!(Send + Sync);
// Maybe support this once min_specialization is supported.
// pub fn trait_cast<'a, Target: Sized + 'static>(
//   source: &'a dyn Any,
//   trait_cast_target: &[TraitcastTarget],
// ) -> Option<&'a Target> {
//   source.downcast_ref::<Target>()
// }
