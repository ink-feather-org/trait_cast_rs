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
