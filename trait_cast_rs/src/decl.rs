/// Declarative macro for automatic implementation of `TraitcastableAny` (and `TraitcastableTo`).
/// Generally not for direct invocation, but rather used by the proc-macro `make_trait_castable`.
///
/// # Usage
/// ```rust
/// make_trait_castable_decl! {
///     SrcStruct1 => (DstTrait1, DstTrait2),
///     SrcStruct2 => (DstTrait3, DstTrait4),
/// }
/// ```

#[macro_export]
macro_rules! make_trait_castable_decl {
  ($($source:ty => ($($target:path),+ $(,)?)),+$(,)?) => {
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
        )+
        impl $crate::TraitcastableAny for $source {
            fn traitcast_targets(&self) -> &[$crate::TraitcastTarget] {
                const TARGETS: &'static [$crate::TraitcastTarget] = &[
                    $(
                        $crate::TraitcastTarget::from::<$source, dyn $target>(),
                    )+
                ];
                TARGETS
            }
        }

    )+
  };
}
