# Array

An array is an ordered collection of other values.

```butter
fruits = ["strawberry", "pineapple"];
```

You can splat arrays, this is used to simulate pushing to array. You can splat as many array as you want.

```butter
fruits <- [..fruits, "banana"];
```

There are other special syntaxes as well. TODO more explanation.

```butter
range = [=0..3];
-- similar to
range = [0, 1, 2];
```

## Indexing

You can access its elements via `[]`.

```butter
first_fruit = &fruits[0];
```

## Length

Arrays have special property `len` that have a value referring to its runtime length.

```butter
std::assert(fruits.len == 3)
```

## Slicing

Through slicing, you can get a portion of the array.

```butter
favorites = &fruits[1..3];
```
