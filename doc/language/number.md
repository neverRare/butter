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

## Arithmetic operations

Pretty straight-forward. The right hand side of division, floor division, and modulo must be non-zero.

- Addition `A + B`
- Subtraction `A - B`
- Multiplication `A * B`
- Division `A / B`
- Floor Division `A // B`
- Modulo `A % B`
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

## Number representation and precision

TODO
