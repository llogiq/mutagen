# Breaking your Rust code for fun & profit

This is a work in progress mutation testing framework. Not all components are there, those that are there aren't finished, but you can see the broad direction it's going to take.

### Mutation Testing

The idea behind mutation testing is to insert changes into your code to see if they make your tests fail. If not, your tests obviously fail to test the changed code.
The difference to line or branch coverage is that those measure if the code under test was *executed*, but that says nothing about whether the tests would have caught any error.

This repo has two components at the moment: A helper library and a procedural macro that mutates your code.

### Contributing

issues and PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) on how to help.
