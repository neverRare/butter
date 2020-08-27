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

AND and OR operators have short-circuiting behavior. When the left hand side of AND operator is false, it immediately evaluates to false without evaluating the right hand side. Same for OR operator with left hand side being true.

Note that NXOR and XOR are just equality operator.
