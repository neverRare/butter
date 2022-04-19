# Boolean

Boolean in butter!

```butter
foo = true;
```

## Logical operators

**Note:** NXOR and XOR are currently not implemented yet. These being simply equality operators, these are only applicable for numbers for now.

- AND `A & B`
- OR `A | B`
- Lazy AND `A && B`
- Lazy OR `A || B`
- NOT `!A`
- NXOR `A == B`
- XOR `A != B`

Lazy operators performs short-circuit. It will not evaluate the right expression when the left expression is already sufficient as the value. For lazy AND, if the left expression is false, it is immediately false. For lazy OR, it is true.

Note that NXOR and XOR are just equality operator.
