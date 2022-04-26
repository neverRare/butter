# Assignment

Assignment replaces a value within a place. Variables that holds the place must be annotated with `mut`.

Assigning a variable itself.

```butter
mut foo = 10;
foo <- 20;
```

Reassign a value of field of a [record].

[record]: ./record.md

```butter
mut foo = (
    prop = 10,
);
foo.prop <- 20;
```

Reassign an element of an [array].

[array]: ./array.md

```butter
mut foo = [10];
foo[0] <- 20;
```

Reassign the referencing value.

[reference]: ./reference.md

```butter
mut foo = 10;
mut bar = &foo;
bar^ <- 20;
```

## Parallel assignment

You can condense multiple assignment into single statement with parallel assignment. Parallel assignment will also simultaneously perform assignment (hence the name parallel), making value swapping possible in single statement.

```butter
mut foo = 10;
mut bar = 20;
foo, bar <- bar, foo;
std.assert(foo == 20);
std.assert(bar == 10);
```

This condensed assignment can be hard to track, we recommend using it only when swapping values.
