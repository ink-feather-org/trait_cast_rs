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
pub trait TraitcastTo<Target: ?Sized> {
  /// Returns true if `Target` is the exact same type as Self
  fn is(&self) -> bool;

  /// Returns true if Self can be converted to a Target
  fn can_be(&self) -> bool;

  fn downcast_ref(&self) -> Option<&Target>;
  #[cfg(feature = "downcast_unchecked")]
  #[doc(cfg(feature = "downcast_unchecked"))]
  fn downcast_ref_unchecked(&self) -> &Target;

  fn downcast_mut(&mut self) -> Option<&mut Target>;
  #[cfg(feature = "downcast_unchecked")]
  #[doc(cfg(feature = "downcast_unchecked"))]
  fn downcast_mut_unchecked(&self) -> &mut Target;

  /// # Errors
  /// In case of the cast being impossible the input is passed back.
  /// Otherwise the box would be dropped.
  #[cfg(feature = "alloc")]
  #[doc(cfg(feature = "alloc"))]
  fn downcast(self: Box<Self>) -> Result<Box<Target>, Box<Self>>;
  #[all(feature = "alloc", feature = "downcast_unchecked")]
  #[cfg(all(feature = "alloc", feature = "downcast_unchecked"))]
  fn downcast_unchecked(self: Box<Self>) -> Box<Target>;
}

macro_rules! implement_with_markers {
  ($($(+)? $traits:ident)*) => {
    impl Debug for dyn Traitcastable $(+ $traits)* {
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
    impl dyn Traitcastable $(+ $traits)* {
      fn get_trait_cast_target<Target: ?Sized + 'static + $($traits +)*>(&self) -> Option<&'static TraitcastTarget> {
        self
        .traitcast_targets()
        .iter()
        .find(|possible| possible.target_type_id == TypeId::of::<Target>())
      }
    }
    impl<Target: ?Sized + 'static + $($traits +)*> TraitcastTo<Target> for dyn Traitcastable $(+ $traits)* {
      default fn is(&self) -> bool {
        false
      }
      default fn can_be(&self) -> bool {
        self.get_trait_cast_target::<Target>().is_some()
      }
      default fn downcast_ref(&self) -> Option<&Target> {
        self.get_trait_cast_target::<Target>()
          .and_then(|target| {
            let fn_ptr: fn(&dyn Traitcastable) -> Option<&Target> =
              unsafe { mem::transmute(target.to_dyn_ref) };
            fn_ptr(self)
          })
      }
      #[cfg(feature = "downcast_unchecked")]
      default fn downcast_ref_unchecked(&self) -> &Target {
        self.downcast_ref().unwrap()
      }

      default fn downcast_mut(&mut self) -> Option<&mut Target> {
        self.get_trait_cast_target::<Target>()
          .and_then(|target| {
            let fn_ptr: fn(&mut dyn Traitcastable) -> Option<&mut Target> =
              unsafe { mem::transmute(target.to_dyn_mut) };
            fn_ptr(self)
          })
      }
      #[cfg(feature = "downcast_unchecked")]
      default fn downcast_mut_unchecked(&self) -> &mut Target {
        self.downcast_mut().unwrap()
      }

      #[cfg(feature = "alloc")]
      default fn downcast(self: Box<Self>) -> Result<Box<Target>, Box<Self>> {
        let raw: *mut Self = Box::into_raw(self) ;
        let raw_ref: &mut Self = unsafe {&mut* raw} ;
        let to_ref: *mut Target = &mut *raw_ref.downcast_mut().ok_or(unsafe {Box::from_raw(raw)})?;
        Ok(unsafe { Box::from_raw(to_ref) })
      }

      #[cfg(all(feature = "alloc", feature = "downcast_unchecked"))]
      default fn downcast_unchecked(self: Box<Self>) -> Box<Target> {
        self.downcast().unwrap()
      }
    }
    impl<Target: Sized + 'static + $($traits +)*> TraitcastTo<Target> for dyn Traitcastable $(+ $traits)* {
      fn is(&self) -> bool {
        <dyn Any>::is::<Target>(self)
      }
      fn can_be(&self) -> bool {
        <dyn Traitcastable as TraitcastTo<Target>>::is(self)
      }
      fn downcast_ref(&self) -> Option<&Target> {
        <dyn Any>::downcast_ref::<Target>(self)
      }
      #[cfg(feature = "downcast_unchecked")]
      fn downcast_ref_unchecked(&self) -> &Target {
        <dyn Any>::downcast_ref::<Target>(self)
      }

      fn downcast_mut(&mut self) -> Option<&mut Target> {
        <dyn Any>::downcast_mut::<Target>(self)
      }
      #[cfg(feature = "downcast_unchecked")]
      fn downcast_mut_unchecked(&self) -> &mut Target {
        <dyn Any>::downcast_mut_unchecked::<Target>(self)
      }

      #[cfg(feature = "alloc")]
      fn downcast(self: Box<Self>) -> Result<Box<Target>, Box<Self>> {
        #[cfg(feature = "downcast_unchecked")]
        if TraitcastTo::<Target>::is(self.as_ref()) { unsafe { Ok(<Box<dyn Any>>::downcast_unchecked(self)) } } else { Err(self) }
        #[cfg(not(feature = "downcast_unchecked"))]
        if TraitcastTo::<Target>::is(self.as_ref()) { Ok(<Box<dyn Any>>::downcast(self).unwrap()) } else { Err(self) }
      }

      #[cfg(all(feature = "alloc", feature = "downcast_unchecked"))]
      fn downcast_unchecked(self: Box<Self>) -> Box<Target> {
        <Box<dyn Any>>::downcast_unchecked::<Target>(self)
      }
    }
  };
}

implement_with_markers!();
implement_with_markers!(Send);
implement_with_markers!(Send + Sync);
