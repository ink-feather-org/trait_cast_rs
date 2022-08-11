use std::any::{Any, TypeId};

struct TraitcastTarget {
    target_type_id: TypeId,
    to_dyn_func: fn(&dyn Traitcastable) -> *const (),
}

trait Traitcastable {
    fn traitcastable_from(&self) -> Vec<TraitcastTarget>;
}

// fn trait_cast<'a, Target: Sized + 'static>(
//   source: &'a dyn Any,
//   trait_cast_target: &[TraitcastTarget],
// ) -> Option<&'a Target> {
//   source.downcast_ref::<Target>()
// }

fn trait_cast<'a, Target: ?Sized + 'static>(
    source: &'a dyn Any,
    trait_cast_target: &[TraitcastTarget],
) -> Option<&'a Target> {
    unsafe {
        let target = trait_cast_target
            .iter()
            .find(|possible| possible.target_type_id == std::any::TypeId::of::<Target>());
        if let Some(target) = target {
            let fn_ptr: fn(&dyn Any) -> Option<&Target> = std::mem::transmute(target.to_dyn_func);
            fn_ptr(source)
        } else {
            None
        }
    }
}
