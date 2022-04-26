# Array

An array is an ordered collection of values with the same type.

```butter
mut fruits = ["strawberry", "pineapple"];
```

## Splat and concat

You can splat arrays, this flattens and combines array to elements or another splatted arrays.

```butter
fruits <- [*>fruits, "banana"];
```

Additionally, you can concatenate, this joins two or more arrays into a single one.

```butter
fruits <- >fruits ++ ["banana"];
```

Splatting and concatenation can serve similar purposes, the 2 examples above does similar task.

**Note:** `>` is a [move] operator

[move]: ./move_and_initialization.md

## Range notation

When defining an array of consecutive integers, you can use range notation. This is often used with [`for`] loops. This uses [range syntax].

[range syntax]: #range-syntax
[`for`]: loop.md#for

```butter
range = [0.<3];
-- similar to
range = [0, 1, 2];
```

## Index

You can access its elements via `[]`.

```butter
first_fruit = &fruits[0];
```

## Length

Arrays have special property `len` that have a value referring to its runtime length.

```butter
std.assert(fruits.len == 3)
```

## Slice

Through slicing, you can get a portion of the array by specifying the indices of the bounds. This uses [range syntax].

[range syntax]: #range-syntax

```butter
favorites = &fruits[1.<3];
```

## Range syntax

You can use `..`, `.<`, `>.`, or `><` for ranges. An angle bracket `<` or `>` means exclusive bound on that side while the period `.` means inclusive. Omitting the bound means there is no bound on that side. The following is an exhaustive list of its possible syntax and its meaning.

| Syntax                    | Meaning       |
| ------------------------- | ------------- |
| `A..B`                    | `A <= x <= B` |
| `A.<B`                    | `A <= x < B`  |
| `A>.B`                    | `A < x <= B`  |
| `A><B`                    | `A < x < B`   |
| `A..` or `A.<`            | `A <= x`      |
| `A>.` or `A><`            | `A < x`       |
| `..B` or `>.B`            | `x <= B`      |
| `.<B` or `><B`            | `x < B`       |
| `..`, `.<`, `>.`, or `><` | -             |
