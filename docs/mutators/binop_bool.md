# `MutatorBinopBool`

## Target Code

expressions containing `&&` and `||`.

## Mutations

1. replacing `&&` with `||`
2. replacing `||` with `&&`

## Limitations

none.

The target code has the same short-circuiting behavior to the original operators: When the right argument is not needed for the value of the mutated or original expression, the right argument is not evaluated.

## Customization

none
