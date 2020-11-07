# Breaking your Rust code for fun & profit
[![Hits](https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2Fllogiq%2Fmutagen&count_bg=%2379C83D&title_bg=%23555555&icon=&icon_color=%23E7E7E7&title=PAGE+VIEWS&edge_flat=false)](https://hits.seeyoufarm.com)

*this is a architecture-preview, not all components are there*

This is a mutation testing framework for Rust code.

## Mutation Testing

A change (mutation) in the program source code is most likely a bug of some kind. A good test suite can detect changes in the source code by failing ("killing" the mutant). If the test suite is green even if the program is mutated (the mutant survives), the tests fail to detect this bug.

Mutation testing is a way of evaluating the quality of a test suite, similar to code coverage.
The difference to line or branch coverage is that those measure if the code under test was *executed*, but that says nothing about whether the tests would have caught any error.

## How `mutagen` works

`mutagen`'s core functionality is implemented via a procedural macro that transforms the source code. Known patterns of code are replaced by mutators with identical behavior unless activated. Activating a mutator at runtime alters its behavior - having the effect of a mutation.

The procedural macro has access the bare AST. Information about inferred types, implemented traits, control flow, data flow or signatures of other functions are not available during the execution of procedural macros. Therefore, the mutations must be possible without additional type-information.

In order to be fast, it is necessary that the compilation of the test suite is performed only once. To achieve this, all mutations are baked into the code once and are selected at runtime via an environment variable. This means the mutations are not allowed to produce code that fails to compile.

This project is basically an experiment to see what mutations we can still apply under those constraints.

## Using mutagen

**Note**: The version of mutagen (`0.2.0`) referenced in this README is not yet released on `crates.io`. To install and use an earlier, released version, you can follow the instructions on [crates.io mutagen crate](https://crates.io/crates/mutagen).

You need Rust nightly to compile the procedural macro.

Add the library `mutagen` as a `dev-dependency` to your `Cargo.toml` referencing this git repository:

```rust
[dev-dependencies]
mutagen = {git = "https://github.com/llogiq/mutagen"}
```

To use the attribute `#[mutate]`, you need to import it.

```rust
#[cfg(test)]
use mutagen::mutate;
```

Now you can advise mutagen to mutate any function or method by prepending `#[cfg_attr(test, mutate)]`. The use of `cfg_attr` ensures the `#[mutate]` attribute will only be active in test mode. The repository contains an example that shows how mutagen could be used.

### Running mutagen

Install `cargo-mutagen`, which can be done by running `cargo install cargo-mutagen`. Run `cargo mutagen` on the project under test for a complete mutation test evaluation.

The mutants can also be run manually: `cargo test` will compile code and write the performed mutations to `target/mutagen/mutations`. This file contains ids and descriptions of possible mutations.
Then, the environment variable `MUTATION_ID` can be used to activate a single mutation as defined by the `mutations` file. The environment variable can be set before calling the test suite, i.e. `MUTATION_ID=1 cargo test`, `MUTATION_ID=2 ..`, etc. For every mutation count at of least one, the test suite should fail

You can run `cargo mutagen -- --coverage` in order to reduce the time it takes to run the mutated code. When running on this mode, it runs the test suite at the beginning of the process and checks which tests are hitting mutated code. Then, for each mutation, instead of running the whole test suite again, it executes only the tests that are affected by the current mutation. This mode is specially useful when the test suite is slow or when the mutated code affects a little part of it.

If you referenced `mutagen` in your cargo.toml via the git repository as noted in the `Using Mutagen` section, you will probably want to install the development version of `cargo-mutagen`. To install the development version, run `cargo install` in the `mutagen-runner` dir of this repository. Running `cargo install --force` might be necessary to overwrite any existing `cargo-mutagen` binary.

## A Word of Warning

Mutagen will change the code you annotate with the `#[mutate]` attribute. This can have dire consequences in some cases. However, functions not annotated with `#[mutate]` will not be altered.

*Do not use `#[mutate]` for code that can cause damage if buggy*. By corrupting the behavior or sanity checks of some parts of the program, dangerous accidents can happen. For example by overwriting the wrong file or sending credentials to the wrong server.

*Use `#[mutate]` for tests only.* This is done by always annotating functions or modules with `#[cfg_attr(test, mutate)]` instead, which applies the `#[mutate]` annotation only in `test` mode. If a function is annotated with plain `#[mutate]` in every mode, the mutation-code is baked into the code even when compiled for release versions. However, when using `mutagen` as `dev-dependency`, adding a plain `#[mutate]` attribute will result in compilation errors in non-test mode since the compiler does not find the annotation.

*Use `mutagen` as `dev-dependency`, unless otherwise necessary.* This ensures that no code from `mutagen` is part of your released library or application.

## Limitations of Mutations

*No mutations will be introduced in `unsafe`-blocks and `unsafe` functions*. Mutations would probably break the some invariantes. Moreover, mutations in unsafe code could lead to undefined behavior that cannot be observed by any testcase.

*`const` and `static` expressions cannot be mutated.* They are evaluated at compile-time and Mutagen can only affect code that can alter its behavior at run-time. Array lengths and global constants are examples of `const` expressions.

*Patterns are cannot mutated.* Mutations are introduced by injecting calls to mutagen-internal functions, which cannot be placed inside patterns.

## Contributing

Issues and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) on how to help.
