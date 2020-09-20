# Number

Numbers in butter! Butter supports integers and floating number, an approximation of real numbers. Butter will automatically use best suited format.

```butter
foo = 12;
bar = 0.5;
hex = 0xff;
bin = 0b11110000;
oct = 0o127;
large_number = 1_000_000;
small_number = 4_e-7;
```

For consistency, literals that are in E-notation are always assumed to be a floating number even if it is integral.

Underscores `_` are optional and used as visual separator. There are few places where it can't be placed:

- In the start of the literal
- Before the base modifier (`b`, `o`, or `x`)
- After the decimal point
- Between the `e` and the sign of the magnitude

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

Modulo internally uses floor division, the result will have the same sign as the right expression.

## Comparison operations

Also pretty straight-forward.

- Equal `A == B`
- Not Equal `A != B`
- Greater than `A > B`
- Greater than or equal to `A >= B`
- Less than `A < B`
- Less than or equal to `A <= B`

## Bitwise operations

Operation in bitwise manner, only works with integer. The right hand side operand of bit-shifts must be non-negative.

- AND `A & B`
- OR `A | B`
- NOT `!<T>A`
- XOR `A ^ B`
- Shift left `A << B`
- Shift right `A >> B`

Bitwise NOT operator is required to be annotated with traditional type, as the operation depends on it. If we let Butter infer it, it may cause ambiguity.

```butter
foo = !<u32>bar;
```

The following annotation are available for bitwise NOT operator.

- `u8`
- `u16`
- `u32`
- `u64`
- `i`
- `i8`
- `i16`
- `i32`
- `i64`
