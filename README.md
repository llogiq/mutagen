# Breaking your Rust code for fun & profit

This is a work in progress mutation testing framework. Not all components are there, those that are there aren't finished, but you can see the broad direction it's going to take.

### Mutation Testing

The idea behind mutation testing is to insert changes into your code to see if they make your tests fail. If not, your tests obviously fail to test the changed code.
The difference to line or branch coverage is that those measure if the code under test was *executed*, but that says nothing about whether the tests would have caught any error.

This repo has two components at the moment: A helper library and a procedural macro that mutates your code.

### How mutagen works

Mutagen works as a procedural macro. This means it only gets to see the code you mark up with the `#[mutate]` annotation, nothing more. It also will
only see the bare AST, no inferred types, no control flow or data flow, unless we analyse them ourselves. But not only that, we want to be *fast*.
This means we want to avoid doing one compile run per mutation, so we try to bake in all mutations into the code once and select them at runtime via
a mutation count. This means we must avoid mutations that break the code so it no longer compiles.

This project is basically an experiment to see what mutations we can still apply under those constraints.

### Contributing

issues and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) on how to help.
