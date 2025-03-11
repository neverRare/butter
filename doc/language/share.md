# Share

Some values can be shareable between different places, these are values that are immutable and never moved.

```butter
share fruits = ["strawberry", "pineapple"];
my_fruits = fruits;
your_fruits = fruits;
-- `fruits`, `my_fruits`, and `your_fruits` shares the same value
```

TODO: clarification for mutable reference, these can be immutable but it cannot be shared.

## Sharable trait

TODO
