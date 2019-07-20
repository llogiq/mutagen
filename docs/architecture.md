# Mutagen Architecture

In `mutagen`, there is a difference between of `mutators` and `mutations`.

1. Mutators are inserted at compile-time by the attribute `#[mutate]`
2. Mutations are introduced at run-time by activating mutators

## Mutators

Mutators are hooks inside the source code can be activated to change the behavior, they do not change the behavior of the code, unless activated. Each mutator is responsible for a single piece in the original code and can be activated for possibly multiple mutations.

### Procedural Macro `#[mutate]`

The attribute `#[mutate]` is a procedural macro that inserts into the program by transforming the source code on the AST-level.

Known patterns of code (e.g. arithmetic operations, boolean logic, loops, ...) get replaced by mutators. The code that describes the transformations to insert each mutator is implemented in the same file with the logic of that mutator.

The compiled test suite contains all mutators.

## Mutations

A mutation occurs when a mutator is activated for a single run of the test suite. By default no mutator is active and calling `cargo test` should run all tests successfully.

A mutator can be activated by setting the environment variable `MUTATION_ID` to a positive number for a single test suite run (e.g. `MUTATION_ID=1 cargo test`). Note that the test suite is not recompiled for each call of `cargo test` if the source code was not changed.

If all tests pass despite the mutation, the mutant "survives". Otherwise, the mutant is "killed". The mutation coverage is the number of mutants that survived.

### Runtime Configuration

The library `mutagen` defines a type `MutagenRuntimeConfig`, which contains the information about which mutation is activated for the current execution of the test suite by querying the `MUTATION_ID` environment variable. The global default config is fetched using the function `MutagenRuntimeConfig::get_default()`, which is already inserted into the source code by the `#[mutate]` attribute.

## Optimistic Mutations

A standard mutation represents a change in the source code such that the changed source code still successfully compiles. A mutation is *optimistic* if the corresponding changed code does not compile when some type-level restrictions are not met.

Since `mutagen` only compiles the test code only once and the procedural macro has no information for the types, mutators with optimistic mutations have to be inserted without the possibility to check if type-level assumptions hold.

If the assumptions on the type are not fulfilled, the mutator panics in order to fail the test suite, since it is not desirable to count such mutants as survivors since they do not represent a valid alteration of the source code. To implement this behavior, the unstable feature `specialization` is used.

Below, there are some examples of optimistic mutators and their type-level assumptions.

### Mutations on arithmetic

Rust's type system allows to write a custom implementation for the operators `+` and `-` independently from each other. In general, if one operator is implemented, the other might not be implemented or the required types differ. However in many contexts, both operators are indeed defined and replacing one by the other would lead to a bug without producing compiler errors.

### Removing negation

Most types that have a implementation for `Not` have `Output = Self`. This is true for the logic and numerical types. Removing the negation will be a valid mutation in most cases.
