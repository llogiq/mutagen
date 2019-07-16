# Breaking your Rust code for fun & profit

*this is a architecture-preview, not all components are there*

This is a mutation testing framework for Rust code.

## Mutation Testing

A change (mutation) in the program source code is most likely a bug of some kind. A good test suite can detect changes in the source code by failing ("killing" the mutant). If the test suite is green even if the program is mutated (the mutant survives), the tests fail to detect this bug.

Mutation testing is a way of evaluating the quality of a test suite, similar to code coverage.
The difference to line or branch coverage is that those measure if the code under test was *executed*, but that says nothing about whether the tests would have caught any error.

## Using mutagen

You need a nightly `rustc` to compile the procedural macro.

Add the library `mutagen` as a `dev-dependency` to your `Cargo.toml`:

```rust
[dev-dependencies]
mutagen = "0.2.0"
```

To use the attribute `#[mutate]`, you need to import it.

```rust
#[cfg(test)]
use mutagen::mutate;
```

Now you can advise mutagen to mutate any function, method, impl, trait impl or whole module (but *not* the whole crate, this is a restriction of procedural macros for now) by prepending `#[cfg_attr(test, mutate)]`. The use of `cfg_attr` ensures the `#[mutate]` attribute will only be active in test mode. The repository contains an example that shows how mutagen could be used.

If the test-suite does not compile, the provided error messages may be unhelpful since the location of the generated code is not set correctly. E.g.:

```
X | #[mutate]
  | ^^^^^^^^^
```  

Calling the test suite with `RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo test` sets the spans accordingly and will produce more helpful error messages.

### Running mutagen

Install `cargo-mutagen`, which can be done by running `cargo install cargo-mutagen`. Run `cargo mutagen` on the project under test for a complete mutation test evaluation.

The mutants can also be run manually: `cargo test` will compile code and write the performed mutations to `target/mutagen/mutations.txt`. This file contains ids descriptions of performed mutations.
Then, the environment variable `MUTATION_ID` can be used to activate a single mutation as defined by `mutations.txt` file. The environment variable can be set before calling the test suite, i.e. `MUTATION_ID=1 cargo test`, `MUTATION_ID=2 ..`, etc. For every mutation count at of least one, the test suite should fail

You can run `cargo mutagen -- --coverage` in order to reduce the time it takes to run the mutated code. When running on this mode, it runs the test suite at the beginning of the process and checks which tests are hitting mutated code. Then, for each mutation, instead of running the whole test suite again, it executes only the tests that are affected by the current mutation. This mode is specially useful when the test suite is slow or when the mutated code affects a little part of it.

If you want the development version of `cargo-mutagen`, run `cargo install` in the runner dir of this repository. Running `cargo install --force` might be necessary to overwrite any existing `cargo-mutagen` binary.

## A Word of Warning

Mutagen will change the code you annotate with the `#[mutate]` attribute. This can have dire consequences in some cases. However, functions not annotated with `#[mutate]` will not be altered.

*Do not use `#[mutate]` with unsafe code.* Doing this would very probably break its invariants. So don't run mutagen against modules or functions containing unsafe code under any circumstances.

*Do not use `#[mutate]` for code that can cause damage if buggy*. By corrupting the behavior or sanity checks of some parts of the program, dangerous accidents can happen. For example by overwriting the wrong file or sending credentials to the wrong server.

*Use `#[mutate]` for tests only.* This is done by always annotating functions or modules with `#[cfg_attr(test, mutate)]` instead, which applies the `#[mutate]` annotation only in `test` mode. If a function is annotated with plain `#[mutate]` in every mode, the mutation-code is baked into the code even when compiled for release versions. However, when using `mutagen` as `dev-dependency`, adding a plain `#[mutate]` attribute will result in compilation errors in non-test mode since the compiler does not find the annotation.

*Use `mutagen` as `dev-dependency`, unless otherwise necessary.* Compiling `mutagen` is time-intensive and library-users should not have to download `mutagen` as a dependency.

## Contributing

Issues and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) on how to help.
