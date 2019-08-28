# Implemented Mutators

`mutagen` provides several mutators out of the box. The table below gives a rough overview of the implemented mutators.

| Mutator | when activated | example |
| -- | -- | -- |
| `binop_add` | changes `+` to `-` | `x+y` -> `x-y` |
| `binop_bool` | changes `&&` to `||` and vice versa | `x||y` -> `x&&y` |
| `binop_cmp` | changes one comparison (`<`, `<=`, `>=`, `>`) to another | `x>y` -> `x<=y` |
| `binop_eq` | changes `==` to `!=` and vice versa | `x==y` -> `x!=y` |
| `lit_int` | mutates an integer literal | `1u8` -> `2u8`  |
| `lit_bool` | inverts bool literals | `false` -> `true` |
| `stmt_call` | removes a call to a method or function | `v.push(1);` -> <no code> |
| `unop_not` | removes the negation `!` | `!x` -> `x` |
