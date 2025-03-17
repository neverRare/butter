# Boolean

Boolean in butter! These are simply [tags](./tag.md).

```butter
foo = @true;
```

## Logical operators

**Note:** NXOR and XOR are currently not implemented yet. These are only applicable for numbers for now.

- AND `A and B`
- OR `A or B`
- NOT `not A`
- NXOR `A == B`
- XOR `A /= B`

`and` and `or` are lazy and performs short-circuit. It will not evaluate the right expression when the left expression is already sufficient as the value. For `and`, if the left expression is `@false`, it is immediately `@false`. For `o`, it is `@true`.

Note that NXOR and XOR are just equality operator.
