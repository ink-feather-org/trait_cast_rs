use crate::make_trait_castable_decl;

const fn _test_empty_trait_cast_targets() {
  struct Woof {}

  make_trait_castable_decl! {
    Woof => (),
  }
}
