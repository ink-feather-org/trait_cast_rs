use crate::{make_trait_castable_decl, TraitcastableAny, TraitcastableAnyInfra};
use alloc::boxed::Box;

const fn _test_empty_trait_cast_targets() {
  struct Woof {}

  make_trait_castable_decl! {
    Woof => (),
  }
}

make_trait_castable_decl! {
    Source => (Print)
}

struct Source(i32);
trait Print {
  fn print(&self) -> i32;
}
impl Print for Source {
  fn print(&self) -> i32 {
    self.0
  }
}

#[test]
fn test_trait_castable() {
  let source = Box::new(Source(5));
  let castable: Box<dyn TraitcastableAny> = source;
  let x: &dyn Print = castable.downcast_ref().unwrap();
  x.print();
}
