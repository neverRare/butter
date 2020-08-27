# Boolean

Boolean in butter!

```butter
foo = true;
```

## Logical operators

- AND `A & B`
- Lazy AND `A && B`
- OR `A | B`
- Lazy OR `A || B`
- NOT `!A`
- NXOR `A == B`
- XOR `A != B`

Lazy AND and OR operators have short-circuiting behavior, this means the right expression only gets evaluated when needed.

Note that NXOR and XOR are just equality operator.
