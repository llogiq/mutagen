# Design Decisions

Several high-level decision have lead to the current design of `mutagen`.

## Opt-in

Mutagen is implemented as a procedural macro, which means that only code annotated with `#[mutate]` is considered for mutations. This is a limitation but also a great feature (see warnings in the `readme`).

## Compile-once

This library is designed to be fast. We cannot afford re-compiling the test suite for every mutation. This means that all mutations have to be baked in at compile-time. This means we must avoid mutations that break the code in a way that it no longer compiles.

## Customization

It should be possible to customize the transformers that should be active and the mutations that are implemented. This is especially necessary in case some transformer leads to compile errors for some input. In this case, `#[mutate]` can still be used by leaving out the incorrect transformers. Leaving out only one or few transformers should be easy.

Users of `mutagen` should be able to customize some mutations. This applies to mutators that produce a large number of mutations (like int literals) but only a few of them are selected by default.
