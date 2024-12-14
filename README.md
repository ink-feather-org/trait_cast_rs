# trait-cast

[![Daily-Nightly](https://github.com/ink-feather-org/trait-cast-rs/actions/workflows/rust_daily_nightly_check.yml/badge.svg)](https://github.com/ink-feather-org/trait-cast-rs/actions/workflows/rust_daily_nightly_check.yml)
[![Rust-Main-CI](https://github.com/ink-feather-org/trait-cast-rs/actions/workflows/rust_main.yml/badge.svg)](https://github.com/ink-feather-org/trait-cast-rs/actions/workflows/rust_main.yml)
[![docs.rs](https://docs.rs/trait-cast/badge.svg)](https://docs.rs/trait-cast)
[![crates.io](https://img.shields.io/crates/v/trait-cast.svg)](https://crates.io/crates/trait-cast)
[![rustc](https://img.shields.io/badge/rustc-nightly-lightgrey)](https://doc.rust-lang.org/nightly/std/)

## Requirements

This crate requires a nightly compiler.

## What can this crate do?

This crate adds the `TraitcastableAny` replacement trait for `Any`.
It closely resembles the `Any` trait for downcasting to a concrete type.
Additionally the `TraitcastableAny` trait allows you to **directly** downcast to other `&dyn Trait`s.

To make this work you must specify all *target* traits you want to be able to downcast to in the `make_trait_castable(Trait1, Trait2, ...)` attribute macro.
This macro can be applied to structs, enums and unions.
It implements the `TraitcastableAny` trait for your struct, enum or union.

Note: No modifications on the *target* traits are necessary. Which allows you to downcast to traits of other libraries you don't control.

## Usage

1. Add the `trait_cast_rs` crate to your `Cargo.toml` and switch to a nightly compiler.

2. Add the `#[make_trait_castable(Trait1, Trait2, ...)]` macro to your struct/enum/union.
    List all traits you eventually want to be able to `downcast` to.
    You must implement all listed traits.

3. Use references to `dyn TraitcastableAny` throughout your code instead of `dyn Any`.

4. Enjoy downcasting to trait objects.

## Example

```rust
#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![feature(ptr_metadata)]
use trait_cast::{
  make_trait_castable, TraitcastableAny, TraitcastableAnyInfra, TraitcastableAnyInfraExt,
};
#[make_trait_castable(Print)]
struct Source(i32);
trait Print {
  fn print(&self);
}
impl Print for Source {
  fn print(&self) {
    println!("{}", self.0);
  }
}

let source = Box::new(Source(5));
let castable: Box<dyn TraitcastableAny> = source;
let x: &dyn Print = castable.downcast_ref().unwrap();
x.print();
```

## EVEN MORE Examples 🔥

Check out the [examples](https://github.com/ink-feather-org/trait-cast-rs/tree/main/trait-cast/examples).

If you want to do something the `make_trait_castable` attribute macro can't handle (like implementing for generic structs - pull requests are welcome)
check out the `manual*.rs` examples.

There is also a decl marco available - check out the `with_decl_macro*.rs` examples.

## Features

* `alloc` - Adds special implementations for `Box`, `Rc` and `Arc`. Default feature.
* `min_specialization` -
  Implements `TraitcastableAny` for `'static` types.
  Even types you don't control.
  However these default implementations of `TraitcastableAny` have no downcast targets.

  It additionally requires the following feature flags in the user code:
  `#![feature(min_specialization)]`
* `downcast_unchecked` - Adds `*_unchecked` variants to the downcast functions.

## Upcasting to the real `Any`

With the `trait_upcasting` rust feature you can even cast any `&dyn TraitcastableAny` to `&dyn Any`.
Alternatively you can list the `Any` trait as a traitcast target.
However it is not possible to cast back to `TraitcastableAny` (pull requests are welcome).

## Authors

[raldone01](https://github.com/raldone01) and [onestacked](https://github.com/chriss0612) are the primary authors and maintainers of this library.

## License

This project is released under either:

- [MIT License](https://github.com/ink-feather-org/trait-cast-rs/blob/main/LICENSE-MIT)
- [Apache License (Version 2.0)](https://github.com/ink-feather-org/trait-cast-rs/blob/main/LICENSE-APACHE)

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
  So this should be `105%` save. ~~As long as `TypeId`s don't collide.~~

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

[`std::any`](https://doc.rust-lang.org/nightly/std/any)

[`std::any::Any`](https://doc.rust-lang.org/nightly/std/any/trait.Any.html)

[`TypeId`](https://doc.rust-lang.org/nightly/std/any/struct.TypeId.html)

[`downcast-rs`](https://crates.io/crates/downcast-rs)

[`intertrait`](https://crates.io/crates/intertrait)

[`traitcast`](https://crates.io/crates/traitcast)

[`traitcast_core`](https://crates.io/crates/traitcast_core)

[`cast_trait_object`](https://crates.io/crates/cast_trait_object)

[`mopa`](https://crates.io/crates/mopa)

[`mopa-maintained`](https://crates.io/crates/mopa-maintained)
