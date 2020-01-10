# How to implement a Mutator

To add a mutator the a few steps are required.

## Name the mutator

Every mutator is named after the code pattern it affects, e.g. mutator `binop_bool` mutates binop-expressions containing the bool-operators `&&` or `||`.

In the following the mutator's name will be denoted by `$name`.

## Implement the mutator

The majority of the mutator's logic is implemented in the file `mutagen-core/src/mutator/mutator_$name.rs`.

Each mutator consists of two functions: transformer and runner.

### Impement the transformer

The transform-function should be named `transform` and placed at the top level of the file. Its signature is like `FnMut(Expr, &SharedTransformInfo, &TransformContext) -> Expr`. The `Expr` is replaced with the general ast-node that the mutator transforms. Currently, `Expr` and `Stmt` are used.

If the input matches the mutator's pattern, the target code should be replaced with a call to the function `run`. How to call this function and its arguments depend on the mutator and the semantics of the original code.

The given `transform_info` is used to register the inserted mutations and get the `mutator_id` to be used for the inserted `run`-function.

### Implement the runner

The `run` method must take the `runtime_config` and the `mutator_id` as an argument.

The first action in this function should be a call to `runtime_config.covered(mutator_id)` to signal that this mutator has been covered.

If the `mutator_id` does not match the `mutation_id` of the runtime, the mutator should execute the code as originally written. Otherwise, the mutated code should be executed. If the mutator implements several mutations, the ids following the `mutator_id` are used to activate these mutations.

## Add the new mutator to to the known mutators

A `pub mod` entry is required in the `mutators.rs` file.

The mutator has to be registered in the `transformer.rs` file:
* The mutator has to be added to the `mk_transformer` function
* The mutator's name has to be added to the `all_transformers` function

## Add tests

Unit tests should be written in the file `mutator_$name`.

The crate `mutagen-selftest` contains integration tests. Each mutator has its own file `test_$name`. The mutator should tested in different situations. For each situation, the expected number of mutations should be given and a test should be written for each mutation-id.

## Document the mutator

The file `docs/mutators.md` documents all mutators and their behavior:

* the mutator's name
* a description of the target code pattern
* a list of possible mutations
* optional: limitations of the mutator, i.e. which code does not get
* optional: supported customization of the mutator
