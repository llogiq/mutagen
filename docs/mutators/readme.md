# Implemented Mutators

`mutagen` provides several mutators out of the box. The table below gives a rough overview of the implemented mutators.

| Mutator | when activated | example |
| -- | -- | -- |
| `MutatorLitInt` | mutates an integer literal | `1u8` -> `2u8`  |
| `MutatorLitBool` | inverts bool literals | `false` -> `true` |
| `MutatorBinopAdd` | changes `+` to `-` | `x+y` -> `x-y` |
| `MutatorUnopNot` | removes the negation `!` | `!x` -> `x` |
