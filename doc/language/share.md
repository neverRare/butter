# Share

Some values are shareable between different places, these values are immutable and never moved.

```butter
fruits = ["strawberry", "pineapple"];
my_fruits = fruits;
your_fruits = fruits;
-- `fruits`, `my_fruits`, and `your_fruits` shares the same value
```

Butter infer such values are shareable when sharing and implicitly copying it makes no logical difference. Butter can use whichever is more efficient.

## Difference with Implicit Copy

Sharing and implicit copy have subtle similarity since they share similar syntax: there is no operator.

Implicitly copyable values may be implicitly copied between mutable to immutable and immutable to mutable places. Whereas with other values, an explicit `clone` or move `>` is needed.
