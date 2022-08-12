use core::{
  any::{Any, TypeId},
  mem,
};

pub struct TraitcastTarget {
  target_type_id: TypeId,
  to_dyn_func: fn(&dyn Traitcastable) -> *const (),
}
impl TraitcastTarget {
  pub const unsafe fn new(
    target_type_id: TypeId,
    to_dyn_func: fn(&dyn Traitcastable) -> *const (),
  ) -> Self {
    Self {
      target_type_id,
      to_dyn_func,
    }
  }
}

pub trait Traitcastable: Any {
  fn traitcastable_from(&self) -> &'static [TraitcastTarget];
}

// Support this once min_specialization is supported.
// pub fn trait_cast<'a, Target: Sized + 'static>(
//   source: &'a dyn Any,
//   trait_cast_target: &[TraitcastTarget],
// ) -> Option<&'a Target> {
//   source.downcast_ref::<Target>()
// }

pub fn trait_cast<'a, Target: ?Sized + 'static>(
  source: &'a dyn Traitcastable,
) -> Option<&'a Target> {
  let target = source
    .traitcastable_from()
    .iter()
    .find(|possible| possible.target_type_id == TypeId::of::<Target>());
  if let Some(target) = target {
    let fn_ptr: fn(&dyn Traitcastable) -> Option<&Target> =
      unsafe { mem::transmute(target.to_dyn_func) };
    fn_ptr(source)
  } else {
    None
  }
}
