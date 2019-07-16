# Customization of `mutagen`

The behavior of `mutagen` and the attribute `#[mutate]` can be customized by adding arguments.

## Configuring the list of transformers

The list of transformers to be run can be specified by adding arguments `only(...)` and `not(...)`. In both cases, a list of transformers is required inside the brackets.

### Examples

```rust
// only mutate int-literals
#[mutate(only(lit_int))]

// only mutate int-literals and `+` operations.
#[mutate(only(lit_int, binop_add))]

// include all mutations except bool literal mutations
#[mutate(not(lit_bool))]
```

## Known Transformers

Each implemented mutator has a corresponding transformer. The following table shows the mapping between names of the transformers and their mutators.

| Transformer name | Mutator |
| `lit_int` | `MutatorLitInt` |
| `lit_bool` | `MutatorLitBool` |
| `binop_add` | `MutatorBinopAdd` |

The details of all mutators are described in their own folder (see: [overview](mutators)).

## arguments for transformers

WIP: implement arguments for transformers

Will probably look like this: some transformers have arguments for their mutators (after from list of mutators)

```
#[mutate(not(early_return), lit_int(+1, =0))]
```
