# Breaking your Rust code for fun & profit

[![Build Status](https://travis-ci.org/llogiq/mutagen.svg?branch=master)](https://travis-ci.org/llogiq/mutagen)
[![Downloads](https://img.shields.io/crates/d/mutagen.svg?style=flat-square)](https://crates.io/crates/mutagen/)
[![Version](https://img.shields.io/crates/v/mutagen.svg?style=flat-square)](https://crates.io/crates/mutagen/)
[![License](https://img.shields.io/crates/l/mutagen.svg?style=flat-square)](https://crates.io/crates/mutagen/)

This is a work in progress mutation testing framework. Not all components are there, those that are there aren't finished, but it's already somewhat usable as of now.

### Mutation Testing

The idea behind mutation testing is to insert changes into your code to see if they make your tests fail. If not, your tests obviously fail to test the changed code.
The difference to line or branch coverage is that those measure if the code under test was *executed*, but that says nothing about whether the tests would have caught any error.

This repo has three components at the moment: The mutagen test runner, a helper library and a procedural macro that mutates your code.

### How mutagen Works

Mutagen works as a procedural macro. This means two things:

1. You'll need a nightly rust toolchain to compile the plugin.
2. it only gets to see the code you mark up with the `#[mutate]` annotation, nothing more.

It also will only see the bare AST, no inferred types, no control flow or data flow, unless we analyse them ourselves. But not only that, we want to be *fast*.  This means we want to avoid doing one compile run per mutation, so we try to bake in all mutations into the code once and select them at runtime via a mutation count. This means we must avoid mutations that break the code so it no longer compiles.

This project is basically an experiment to see what mutations we can still apply under those constraints.

### A Word of Warning

mutagen will change the code you annotate with the `#[mutate]` attribute. As long as you use it with safe code, all is well. However, running mutagen against unsafe code will very probably break its invariants, with possible dire consequences. So don't run mutagen against modules containing unsafe code under any circumstances.

### Using mutagen

Again, remember you need a nightly `rustc` to compile the plugin. Add the plugin and helper library as a dev-dependency to your `Cargo.toml`:

```rust
[dev-dependencies]
mutagen = "0.1.0"
mutagen-plugin = "0.1.0"
```

Now, you can add the plugin to your crate by prepending the following:

```rust
#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(mutagen_plugin))]
#![feature(custom_attribute)]

#[cfg(test)]
extern crate mutagen;
```

Now you can advise mutagen to mutate any function, method, impl, trait impl or whole module (but *not* the whole crate, this is a restriction of procedural macros for now) by prepending:

```rust
#[cfg_attr(test, mutate)]
```

This ensures the mutation will only be active in test mode.

### Running mutagen

Install `cargo-mutagen`. Run `cargo mutagen` on the project under test.

If you want the development version, run `cargo install` in the runner dir.

If you want to do this manually you can run `cargo test` as always, which will mutate your code and write a list of mutations in `target/mutagen/mutations.txt`. For every mutation, counting from one, you can run the test binary with the environment variable `MUTATION_COUNT=1 target/debug/myproj-123456`, `MUTATION_COUNT=2 ..`, etc.

You can run `cargo mutagen -- --coverage` in order to reduce the time it takes to run the mutated code. When running on this mode, it runs the testsuite at the beginning of the process and checks which tests are hitting mutated code. Then, for each mutation, instead of running the whole testsuite again, it executes only the tests that are affected by the current mutation. This mode is specially useful when the testsuite is slow or when the mutated code affects a little part of it.

### Contributing

Issues and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) on how to help.
