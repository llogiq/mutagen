# `MutatorUnopNot`

## Target Code

`!`-expressions like `!done`

## Mutations

1. removing the negation (replacing `!x` with `x`)

## Limitations

This is a optimistic mutator. For some types the output type may be too different from the input type such that the input type cannot be converted to it via `Into` without calling the negation.

## Customization

none
