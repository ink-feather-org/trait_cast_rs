#![no_std]
#![deny(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks, clippy::pedantic, clippy::nursery)]
#![allow(incomplete_features)]
#![feature(
  const_type_id,      // Needed to enable `TraitcastTarget::create` to be const
  const_type_name,    // Needed for `Debug` implementation
  trait_upcasting,    // Needed to avoid reimplementing Any
  const_mut_refs,     // Needed since arguments to `TraitcastTarget::create` need a function pointer with &mut argument and return type.
  min_specialization, // Needed to unify the interface between downcast and traitcast (could be avoided with !Trait bounds or trait generics)
  doc_cfg             // For nicer Docs
)]
#![cfg_attr(feature = "downcast_unchecked", feature(downcast_unchecked))]
// #![cfg_attr(feature = "const_cmp_type_id", feature(const_cmp_type_id))] // Needed to impl const cmp for `TraitcastTarget` FIXME: Once const_cmp_type_id lands.
#![cfg_attr(feature = "const_cmp_type_id", feature(const_trait_impl))] // Needed to use the const_sort crate.
#![cfg_attr(feature = "const_sort", feature(const_cmp))] // FIXME: Remove once `const_cmp_type_id` lands
/*!
## Requirements

This crate requires a nightly compiler.1

## What can this crate do?

This crate adds the `TraitcastableAny` replacement trait for `Any`.
It closely resembles the `Any` trait for downcasting to a concrete type.
Additionally the `TraitcastableAny` trait allows you to **directly** downcast to other `&dyn Trait`s.

To make this work you must specify all *target* traits you want to be able to downcast to in the `make_trait_castable(Trait1, Trait2, ...)` attribute macro.
This macro can be applied to structs, enums and unions.
It implements the `TraitcastableAny` trait for your struct, enum or union.

Note: No modifications on the *target* traits is necessary. Which allows you to `downcast` to traits of other libraries you don't control.

## Usage

1. Add the `trait_cast_rs` crate to your `Cargo.toml` and switch to a nightly compiler.

2. Add the `#[make_trait_castable(Trait1, Trait2, ...)]` macro to your struct/enum/union.
    List all traits you eventually want to be able to `downcast` to.
    You must implement all listed traits.

3. Use references to `dyn TraitcastableAny` throughout your code instead of `dyn Any`.

4. Enjoy downcasting to trait objects.

## Example

```rust
# #![cfg_attr(feature = "min_specialization", feature(min_specialization))]
use trait_cast_rs::{
  make_trait_castable, TraitcastableAny, TraitcastableAnyInfra, TraitcastableAnyInfraExt,
};
#[make_trait_castable(Print)]
struct Source(i32);
trait Print {
  fn print(&self);
}
impl Print for Source {
  fn print(&self) {
    println!("{}", self.0)
  }
}

let source = Box::new(Source(5));
let castable: Box<dyn TraitcastableAny> = source;
let x: &dyn Print = castable.downcast_ref().unwrap();
x.print();
```

## EVEN MORE Examples 🔥

Check out the [examples](https://github.com/raldone01/trait_cast_rs/tree/main/trait_cast_rs/examples).

If you want to do something the `make_trait_castable` attribute macro can't handle (like implementing for generic structs - pull requests are open)
check out the `manual*.rs` examples.

There is also a decl marco available - check out the `with_decl_macro*.rs` examples.

## Good to know

With the `trait_upcasting` feature you can even cast any `&dyn TraitcastableAny` to `&dyn Any`.
Alternatively you can list the `Any` trait as a traitcast target.

Consider enabling the `min_specialization` crate trait.
It makes all `'static` types into `TraitcastableAny`s.
Even types you don't control.
However the default implementation of `TraitcastableAny` has no downcast targets.

## Premature Optimization

The order of the parameters for `make_trait_castable` determines the lookup order.
So you should order them according to the `downcast` frequency.
With the most frequent traits first.

## Authors

[raldone01](https://github.com/raldone01) and [onestacked](https://github.com/chriss0612) are the primary authors and maintainers of this library.

## License

This project is released under either:

- [MIT License](https://github.com/raldone01/trait_cast_rs/blob/main/LICENSE-MIT)
- [Apache License (Version 2.0)](https://github.com/raldone01/trait_cast_rs/blob/main/LICENSE-APACHE)

at your choosing.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## How it works

I will give you a quick rundown of our *internal* operations: 💦

Compile time:

1. Add a `casting` function for every downcast path to the concrete type.
    This function gets a `dyn TraitcastableAny`, which it then downcasts to a concrete type using `Any` in the background.
    In the last step it casts the concrete type to the wanted trait object and returns it.

2. Add a `traitcast_targets` function that returns a const slice of (`typeid`, transmuted *casting* function ptr).

Runtime:

1. Get targets array
2. Find the target `typeid`
3. Transmute function pointer back to original type
4. Call the function pointer to get the wanted trait object
5. Return it
6. 💲 Profit 💲

## SAFETY 🏰

* The unchecked variants of the `downcast` function all use unsafe - expectedly.
* The only other use of unsafe is the transmutation of function pointers.
  However when they are called they are transmuted back to their original type.
  So this should be `105%` save. ~~As long as `typeid`s don't collide.~~

## Alternatives (~~and why our crate is the best~~)

This alternatives section is not exhaustive for a more objective/detailed comparison
see the alternatives section of [cast_trait_object](https://crates.io/crates/cast_trait_object#Alternatives).

* [mopa](https://crates.io/crates/mopa):
    Had its last update 6 years ago.
    Has some unresolved [unsoundness issues](https://github.com/chris-morgan/mopa/issues/13).
    Also requires modifications on traits themselves while we just modify the struct/enum/union (see note above).
* [mopa-maintained](https://crates.io/crates/mopa-maintained):
    Might have fixed some issues but still has an old code base with just a version bump.
* [traitcast](https://crates.io/crates/traitcast):
    Has no readme on [crates.io](https://crates.io/).
    Uses a GLOBAL REGISTRY with `lazy_static`.
    To be fair it allows you to use the default `Any` and doesn't require nightly.

TODO: Remove this section once our last update is 6 years old.

### Links

[`std::any`](https://doc.rust-lang.org/std/any)

[`std::any::Any`](https://doc.rust-lang.org/std/any/trait.Any.html)

[`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html)

[`downcast-rs`](https://crates.io/crates/downcast-rs)

[`intertrait`](https://crates.io/crates/intertrait)

[`traitcast`](https://crates.io/crates/traitcast)

[`traitcast_core`](https://crates.io/crates/traitcast_core)

[`mopa`](https://crates.io/crates/mopa)

[`mopa-maintained`](https://crates.io/crates/mopa-maintained)

*/

#[cfg(feature = "alloc")]
extern crate alloc;

mod trait_cast;
pub use trait_cast::*;

mod decl;

pub use trait_cast_impl_rs::make_trait_castable;

#[cfg(test)]
mod test;
