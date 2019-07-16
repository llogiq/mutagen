# `MutatorBinopAdd`

## Target Code

`+` expressions like `a+y`

## Mutations

1. replacing `+` with `-`

## Limitations

This is a optimistic mutator. Not for every type, the trait `Sub` is implemented with the corresponding right-hand-side and the corresponding output

## Customization

Customization is WIP
Changing the `+` to the other binary operations `*`, `/` and `%` as well as the bit-wise operations are valid optimistic mutations.
