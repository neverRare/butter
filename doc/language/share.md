# Share

Some values are shareable between different places, these are values that are immutable and never moved.

```butter
fruits = ["strawberry", "pineapple"];
my_fruits = fruits;
your_fruits = fruits;
-- `fruits`, `my_fruits`, and `your_fruits` shares the same value
```

TODO: clarification for mutable reference, these can be immutable but it cannot be shared.

## Difference with Implicit Copy

Sharing and implicit copy have subtle similarity since they share similar syntax: there is no operator.

Implicitly copyable values may be implicitly copied between mutable to immutable and immutable to mutable places. Whereas with other values, an explicit `clone` or move `>` is needed.

## Sharable trait

TODO
