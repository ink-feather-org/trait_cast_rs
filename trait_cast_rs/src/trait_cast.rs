use core::{
  any::{Any, TypeId},
  mem,
};

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
  fn traitcastable_from(&self) -> &'static [TraitcastTarget];
}

// Maybe support this once min_specialization is supported.
// pub fn trait_cast<'a, Target: Sized + 'static>(
//   source: &'a dyn Any,
//   trait_cast_target: &[TraitcastTarget],
// ) -> Option<&'a Target> {
//   source.downcast_ref::<Target>()
// }

pub fn trait_cast<Target: ?Sized + 'static>(source: &dyn Traitcastable) -> Option<&Target> {
  let target = source
    .traitcastable_from()
    .iter()
    .find(|possible| possible.target_type_id == TypeId::of::<Target>());

  target.and_then(|target| {
    let fn_ptr: fn(&dyn Traitcastable) -> Option<&Target> =
      unsafe { mem::transmute(target.to_dyn_func) };
    fn_ptr(source)
  })
}
