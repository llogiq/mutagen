# Test Plan

`mutagen`'s logic is tested using unit tests, integration tests and an example-crate.

## Unit tests of mutators

The behavior of all mutators is tested using traditional unit tests in the corresponding modules. The mutators are run with run-time configurations constructed by the test itself.

## Tests of `#[mutate]`

The behavior of the `#[mutate]` attribute for single mutators is tested inside the crate `mutagen-selftest`.

### Test Isolation

Instead of having a program-wide unique id per mutation, each function can have their own local mutation ids starting from one by writing adding `conf = local` to the arguments of `#[mutate]`. This ensures that the mutation ids are independent for each test. Moreover, only one mutator is enabled in each test by adding `mutators = only(...)` to the mutagen arguments.

### Exhaustive Testing

Each test sets the `mutation_id` for the single test run and runs test code. This enables exhaustive testing of all mutations within a single run of the test suite and without dependency on external environment variables.

For every mutator, it is tested whether all its mutations actually produce the expected deviation from standard behavior and that they have no effect on the code unless activated.

To ensure that the expected number of mutations are generated, the argument `expected_mutations` is added to the configuration. This includes a check that exactly that many mutations are generated by the function nuder test

### Implementation Details

The crate `mutagen-selftest` has dependencies to `mutagen` and `mutagen-core` and uses the libraries similar to other creates.

Setting the `mutation_id` during test is possible via special functions that mutate the global run-time configuration and are only available when enabling the feature `self_test` of `mutagen-core`. The feature `self_test` is not supposed to be used by users of `mutagen`.

### Example

Typically, a complete test of some feature looks like this
```rust
mod test_x {
    // only enable mutator `xyz`
    // assert that 2 mutations will be generated
    #[mutate(conf = local(expected_mutations = 2), mutators = only(xzy))]
    pub fn x() {
        // function to mutate
    }

    #[test]
    pub fn x_inactive() {
        test_without_mutation( || {
            // test and assert on `x()` where no mutations have been performed
        })
    }
    #[test]
    pub fn x_active() {
        test_with_mutation_id(1, || {
            // test and assert that the correct mutation has been performed in `x()`
        })
    }

    // more tests with other mutation ids, if more than one mutation has been performed
}
```

## Tests of an example crate

With the crate `example/simple` of this repository, end-to-end tests of mutagen can be performed. The binary `cargo-mutagen` can be run on this crate to test the functionality of all parts of the system, including building a mutation report.

The test can be executed with the command.

```bash
cargo run -p cargo-mutagen
```
