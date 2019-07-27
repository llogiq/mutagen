# `MutatorBinopCmp`

## Target Code

expressions that compare two values:

* `x < y`
* `x <= y`
* `x >= y`
* `x > y`

## Mutations

1. replacing the comparison with any of the other three

## Limitations

none - all operations are defined by the trait `PartialOrd`

## Customization

none
