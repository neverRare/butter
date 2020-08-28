# Boolean

Boolean in butter!

```butter
foo = true;
```

## Logical operators

- AND `A && B`
- OR `A || B`
- NOT `!A`
- NXOR `A == B`
- XOR `A != B`

AND and OR operators have short-circuiting behavior, this means the right expression only gets evaluated when needed, when the left expression of AND operator is false, the whole expression is immediately false, same for OR operator with true.

Note that NXOR and XOR are just equality operator.
