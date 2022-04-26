# Tuple

A Tuple is an ordered collection of a fixed number of values with variety of types. These can be thought of as a [record] but keyed with orders. These values are called field or tuple field.

[record]: ./record.md

```butter
color = (15, 120, 211);
position = (10, 20);
```

We can splat multiple tuples, there can be only at most one splat.

```butter
val1 = (15, 120);
val2 = (*val1, 211)
```

## Field access

**Note:** This is not implemented yet

You can access an element of tuple with `.`.

```butter
color = (15, 120, 211);
red = color.0;
```
