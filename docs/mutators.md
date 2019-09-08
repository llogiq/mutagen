# Implemented Mutators

`mutagen` provides several mutators. The document gives a rough overview of the implemented mutators.

## lit_bool

### Target Code

bool literals `true` and `false`

### Mutations

1. negating the value of the literal

## lit_int

### Target Code

Integer literals like `0`, `1u8`, `5isize`.

Byte-literals like `b'a'` are not mutated by this mutator.

### Mutations

1. replacing the literal with a different literal

### Limitations

* literals cannot be mutated into negative numbers
* literals with a value that does not fit into `u128` are not mutated

### Customization

Customization is WIP

## unop_not

### Target Code

`!`-expressions, like `!done`

### Mutations

1. removing the negation, i.e. replacing `!x` with `x`

### Limitations

This is a optimistic mutator. For some types the output type of the negation may be too different from the input type.

such that the input type cannot be converted to it via `Into` without calling the negation.

## binop_bool

### Target Code

expressions containing `&&` and `||`.

### Mutations

1. replacing `&&` with `||`
2. replacing `||` with `&&`

### Limitations

none.

The target code has the same short-circuiting behavior to the original operators: When the right argument is not needed for the value of the mutated or original expression, the right argument is not evaluated.

## binop_cmp

### Target Code

expressions that compare two values:

* `x < y`
* `x <= y`
* `x >= y`
* `x > y`

### Mutations

1. replacing the comparison with any of the other three

## binop_eq

### Target Code

`==`-expressions, like `x == y`

### Mutations

1. replacing `==` with `!=`
1. replacing `!=` with `==`

## binop_add

### Target Code

`+`-expressions, like `a+y`

### Mutations

1. replacing `+` with `-`

### Limitations

This is a optimistic mutator. Not for every type, the trait `Sub` is implemented with the corresponding right-hand-side and the corresponding output

### Customization

Customization is WIP

Changing the `+` to the other binary operations `*`, `/` and `%` as well as the bit-wise operations are valid optimistic mutations.

## stmt_call

### Target Code

Statements that call a single function or method

### Mutations

1. removing the call to the function or method