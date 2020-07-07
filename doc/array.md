# Array

An array is an ordered collection of other values.

```butter
fruits = ["strawberry", "pineapple"];
```

You can splat arrays or a reference to it, this is used to simulate pushing to array. You can splat as many array as you want. Be catious that splatting a reference to an array will splat references to its element.

```butter
fruits <- [..fruits, "banana"];
```

There are other special syntaxes as well. TODO more explanation.

```butter
repeated = ["thing"; 3];
-- this is similar to
repeated = ["thing", "thing", "thing"];

range = [=0..3];
-- similar to
range = [0, 1, 2];
```

## Indexing

You can access its elements via `[]`. You can also index on reference to array.

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

When slicing a reference to array, it returns a reference to an array with fixed length.
