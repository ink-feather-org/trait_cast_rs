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
          const METADATA: ::core::ptr::DynMetadata<dyn $target> = {
            let ptr: *const $source = ::core::ptr::from_raw_parts(::core::ptr::null(), ());
            let ptr: *const dyn $target = ptr as _;

            ptr.to_raw_parts().1
          };
        }
      )*
      // Safety:
      // All returned `TraitcastTarget` are valid for $source
      unsafe impl $crate::TraitcastableAny for $source {
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
            let mut targets : [$crate::TraitcastTarget; TARGETS_LEN] = [
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
    $targets.const_sort_unstable_by_key($crate::TraitcastTarget::target_type_id);
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
    fn find_traitcast_target(
      &self,
      target: ::core::any::TypeId,
    ) -> Option<&$crate::TraitcastTarget> {
      use ::const_sort_rs::ConstSliceSortExt;
      let targets = self.traitcast_targets();
      debug_assert!(
        targets.const_is_sorted_by(|l, r| l.target_type_id().partial_cmp(&r.target_type_id()))
      );

      targets
        .binary_search_by_key(&target, $crate::TraitcastTarget::target_type_id)
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
