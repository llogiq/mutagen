# Customization of `mutagen`

The behavior of `mutagen` and the attribute `#[mutate]` can be customized by adding arguments.

## Configuring the list of mutators

The list of active mutators for a function to be run can be specified by adding arguments `only(...)` and `not(...)`. In both cases, a list of mutators is required inside the brackets.

The details of all mutators are described in their own folder (see: [overview](mutators)).

### Examples

```rust
// only mutate int-literals
#[mutate(only(lit_int))]

// only mutate int-literals and `+` operations.
#[mutate(only(lit_int, binop_add))]

// include all mutations except bool literal mutations
#[mutate(not(lit_bool))]
```

## WIP: arguments for mutators

Will probably look like this: some mutators have arguments, given after the list of mutators

```
#[mutate(not(early_return), lit_int(+1, =0))]
```
