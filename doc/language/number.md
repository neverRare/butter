# Number

Numbers in butter!

```butter
foo = 12;
bar = 0.5;
```

## Arithmetic operations

Pretty straight-forward. The right hand side of division, floor division, and modulo must be non-zero.

- Addition `A + B`
- Subtraction `A - B`
- Multiplication `A * B`
- Division `A / B`
- Floor Division `A // B`
- Modulo `A % B`
- Unary plus `+A`
- Unary minus `-B`

## Comparison operations

Also pretty straight-forward.

- Equal `A == B`
- Not Equal `A != B`
- Greater than `A > B`
- Greater than or equal to `A >= B`
- Less than `A < B`
- Less than or equal to `A <= B`

## Bitwise operations

Operation in bitwise manner, only works with integer. The right hand side operand of bitshifts must be non-negative.

- And `A & B`
- Or `A | B`
- Not `~[T] A` (requires to be annotated with traditional type)
- Xor `A ^ B`
- Shift left `A << B`
- Shift right `A >> B`

Bitwise not operator is required to be annotated with traditional type, as the operation depends on it. If we let Butter infer it, it may cause inconsistency.

```butter
foo = ~[u32] bar;
```
