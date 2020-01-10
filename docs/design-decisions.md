# Design Decisions

Several high-level decision have lead to the current design of `mutagen`.

## Opt-in

Mutagen is implemented as a procedural macro, which means that only code annotated with `#[mutate]` is considered for mutations. This is a limitation but also a great feature (see warnings in the `readme`).

## Compile-once

This library is designed to be fast. We cannot afford re-compiling the test suite for every mutation. This means that all mutations have to be baked in at compile-time. This means we must avoid mutations that break the code in a way that it no longer compiles.

## Procedural Macro

The functionality to inject mutators into the source code is implemented with a procedural macro. Since the stabilization of the [`proc_macro`](https://doc.rust-lang.org/stable/proc_macro/index.html) crate and the development of the libraries [`proc-macro2`](https://crates.io/crates/proc-macro2), [`syn`](https://crates.io/crates/syn) and [`quote`](https://crates.io/crates/quote) writing procedural macros has become more popular and a lot easier.

The input of the procedural macro is the parsed source that was annotated with the attribute `#[mutate]`. It does not have access to any information about inferred types, implemented traits and signatures of other functions. Since the test-suite is compiled only once, no mutator is allowed to produce code that fails to compile. This restricts the possible mutations.

## Customization

It should be possible to customize the list of mutators for each method. This is especially necessary in case some mutators leads to compile errors for some input. Omitting a single or few mutators is possible by giving a blacklist. Equivalently, a whitelist of mutators can be given.

Users of `mutagen` should be able to customize some mutations. This is especially relevant for mutators that can produce a large number of mutations (like int literals) but only a few of them are selected by default.
