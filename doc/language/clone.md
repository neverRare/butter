# Clone

Clone creates a deep copy of a value, with exception to immutable reference where it would be shallowly copied. Any mutation within the original value won't affect the cloned value and vice versa.

```butter
mut foo = 10;
bar = clone foo;
foo <- 20;
std.assert(bar == 10);
```

You can't clone mutable references and functions.

## Implicit Copy

Some types can be implicitly copied. These are numbers, booleans, and immutable references of any type. The example above can be rewritten as:

```butter
mut foo = 10;
bar = foo;
foo <- 20;
std.assert(bar == 10);
```
