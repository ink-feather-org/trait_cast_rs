Examples
========

Proc-macro flavour 🤖
---------------------

Most of the time you will want to use the proc-macro.

* [`with_proc_macro`](with_proc_macro.rs) || [`with_proc_macro_enum`](with_proc_macro_enum.rs): Simplest use case for this crate.
* [`with_proc_macro_gen`](with_proc_macro_gen.rs): Shows that the proc-macro supports casting to concrete generic traits.
* [`with_proc_macro_mut`](with_proc_macro_mut.rs): Shows how to downcast mutably.

Decl-macro flavour 🖨️
---------------------

The decl-macro is slightly more powerful than the proc-macro.
It adds support for concrete generic structs/enums/unions.

* [`with_decl_macro`](with_decl_macro.rs): Simplest use case for the decl-macro.

  Note: Also possible with the proc-macro. See [`with_proc_macro`](with_proc_macro.rs).

* [`with_decl_macro_gen`](with_decl_macro_gen.rs): Shows that the decl-macro supports casting to concrete generic traits.

  Note: Also possible with the proc-macro. See [`with_proc_macro_gen`](with_proc_macro_gen.rs).

* [`with_decl_macro_gen_struct`](with_decl_macro_gen_struct.rs): Shows that the decl-macro supports concrete generic structs/enums/unions.

  Note: This is not possible with the proc-macro.

Manual flavour 📝
-----------------

The manual method requires you to implement the `TraitcastableAny` and `TraitcastableTo` traits yourself.

It is possible to support generic implementations for structs/enums/unions.
Neither proc-macro nor decl-macro support this.

It allows you to optimize the performance by implementing `find_traitcast_target` yourself.

TODO: It would be ideal to sort the targets array at compile time and then do a binary search for the correct target. This however is not yet implemented.

* [`manual`](manual.rs): Simplest manual implementation.
* [`manual_gen_struct`](manual_gen_struct.rs): Shows how to support generic implementations for structs/enums/unions.
