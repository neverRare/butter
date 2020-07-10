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

- Greater than `A > B`
- Greater than or equal to `A >= B`
- Less than `A < B`
- Less than or equal to `A <= B`

## Bitwise operations

Only works with integer, the right hand side operand of bitshifts must be non-negative.

- And `A & B`
- Or `A | B`
- Not `~[T] A` (requires to be annotated with traditional type)
- Xor `A ^ B`
- Shift left `A << B`
- Shift right `A >> B`

## Operator Annotation

Butter can infer rich numeric types and use this to determine the physical size of these numbers. Most operators work consistently across different sizes, one exception is the bitwise not operator. We need to annotate it with traditional type.

```butter
foo = ~[u16] bar;
```

Additionally, Butter adds runtime overflow gaurd to few operators whenever possible. You can use these kind of annotation to allow overflow.

```butter
foo <- foo +[u64] 1;
```

The following are operators that can cause such error, and hence, can be annotated.

- Addition
- Subtraction, underflow can also happen.
- Multiplication
- Bitshift left

For now, annotation only works with integers.

## Traditional integer types

| Size in bits | Unsigned | Signed |
| ------------ | -------- | ------ |
| 8            | `u8`     | `i8`   |
| 16           | `u16`    | `i16`  |
| 32           | `u32`    | `i32`  |
| 64           | `u64`    | `i64`  |
