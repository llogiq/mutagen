# Customization of `mutagen`

The behavior of `mutagen` and the attribute `#[mutate]` can be customized by adding arguments.

## Configuring the list of mutators

The list of active mutators for a function to be run can be specified by adding arguments `mutators = only(...)` and `not(...)`. In both cases, a list of mutators is required inside the brackets.

The details of all mutators are described in their own folder (see: [overview](mutators)).

### Examples

```rust
// only mutate int-literals
#[mutate(mutators = only(lit_int))]

// only mutate int-literals and `+` operations.
#[mutate(mutators = only(lit_int, binop_num))]

// include all mutations except bool literal mutations
#[mutate(mutators = not(lit_bool))]
```

## WIP: arguments for mutators

Will probably look like this: some mutators have arguments, given after the list of mutators

```
#[mutate(not(early_return), lit_int(+1, =0))]
```
