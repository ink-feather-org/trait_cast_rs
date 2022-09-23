/// Declarative macro for automatic implementation of `TraitcastableAny` (and `TraitcastableTo`).
/// Generally not for direct invocation, but rather used by the proc-macro `make_trait_castable`.
///
/// Syntax: `<concrete struct/enum/union> => (<target traits>, ...)`
///
/// # Usage
/// ```no_build
/// make_trait_castable_decl! {
///     SrcStruct1 => (DstTrait1, DstTrait2),
///     SrcStruct2 => (DstTrait3, DstTrait4),
/// }
/// ```
#[macro_export]
macro_rules! make_trait_castable_decl {
  ($($source:ty => ($($target:path),* $(,)?)),+$(,)?) => {
    $(
      $(
        impl $crate::TraitcastableTo<dyn $target> for $source {
          fn to_dyn_ref(input: &dyn $crate::TraitcastableAny) -> Option<&(dyn $target + 'static)> {
            let casted: Option<&Self> = input.downcast_ref();
            casted.map(|selv| selv as &dyn $target)
          }

          fn to_dyn_mut(input: &mut dyn $crate::TraitcastableAny) -> Option<&mut (dyn $target + 'static)> {
            let casted: Option<&mut Self> = input.downcast_mut();
            casted.map(|selv| selv as &mut dyn $target)
          }
        }
      )*
      impl $crate::TraitcastableAny for $source {
        fn traitcast_targets(&self) -> &[$crate::TraitcastTarget] {
          #[allow(clippy::unused_unit)]
          const TARGETS_LEN: usize = {
            let a:&[()] = &[$({
              let _: &dyn $target;
              ()
            },)*];
            a.len()
          };
          const TARGETS: [$crate::TraitcastTarget; TARGETS_LEN] = {
            #[allow(unused_mut)]
            let mut targets = [
              $(
                $crate::TraitcastTarget::from::<$source, dyn $target>(),
              )*
            ];
            $crate::maybe_sort!(targets);
            targets
          };
          &TARGETS
        }
        $crate::maybe_impl_bin_search!();
      }
    )+
  };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "const_sort")]
macro_rules! maybe_sort {
  ($targets:ident) => {{
    use const_sort_rs::ConstSliceSortExt;
    //$targets.const_sort_unstable_by_key($crate::TraitcastTarget::target_type_id); // FIXME: once `const_cmp_type_id` lands
    $targets.const_sort_unstable_by($crate::TraitcastTarget::cmp_by_target_type_id);
  }};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "const_sort"))]
macro_rules! maybe_sort {
  ($targets:ident) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "const_sort")]
macro_rules! maybe_impl_bin_search {
  () => {
    unsafe fn find_traitcast_target(
      &self,
      target: ::core::any::TypeId,
    ) -> Option<&$crate::TraitcastTarget> {
      use ::const_sort_rs::ConstSliceSortExt;
      let targets = self.traitcast_targets();
      debug_assert!(targets
        .const_is_sorted_by(|l, r| Some($crate::TraitcastTarget::cmp_by_target_type_id(l, r))));

      targets
        .binary_search_by(|probe| $crate::cmp_type_id(probe.target_type_id(), target)) // FIXME: once `const_cmp_type_id` lands
        .map(|idx| &targets[idx])
        .ok()
    }
  };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "const_sort"))]
macro_rules! maybe_impl_bin_search {
  () => {};
}
