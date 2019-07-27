# `MutatorLitInt`

## Target Code

Integer literals like `0`, `1u8`, `5isize`

## Mutations

1. replacing the literal with a different literal

## Limitations

* literals cannot be mutated into negative numbers
* literals with a value that does not fit into an `u64` are not mutated

## Customization

Customization is WIP
Changing the literal to all other values would be valid mutations.
