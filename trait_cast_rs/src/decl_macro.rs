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
            let ptr: *const $source = ::core::ptr::null::<$source>();
            let ptr: *const dyn $target = ptr as _;

            ptr.to_raw_parts().1
          };
        }
      )*
      // Safety:
      // All returned `TraitcastTarget`s are valid for $source
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
            targets
          };
          &TARGETS
        }
      }
    )+
  };
}
