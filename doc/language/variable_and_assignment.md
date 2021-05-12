# Variable

Variable holds a value.

## Declaration

Variables are declared via `=`.

```butter
foo = 10;
bar = false;
```

Variables can shadow previously declared variable with the same name, either on the same or on upper scope.

```butter
foo = 10;
{
    foo = false;
    std.assert(foo == false);
}
std.assert(foo == 10);
foo = 20;
std.assert(foo == 20);
foo = foo == 20;
std.assert(foo == true);
```

## Mutable variable

Variables are immutable by default. You can opt to mutable variable by prefixing it with `mut`.

```butter
mut foo = 10;
```

## Assignment

Assignment is a way (and the only way) to mutate such values with `<-` operator.

```butter
mut foo = 10;
foo <- 20;
```

Reassign a value of field of a [struct].

[struct]: struct.md

```butter
mut foo = (
    prop = 10,
);
foo.prop <- 20;
```

Reassign an element of an [array].

[array]: array.md

```butter
mut foo = [10];
foo[0] <- 20;
```

Reassign the referencing value.

[reference]: reference.md

```butter
mut foo = 10;
mut bar = &mut foo;
bar^ <- 20;
```

## Parallel assignment

You can condense multiple assignment into single statement with parallel assignment. Parallel assignment will also simultaneously perform assignment, making value swapping possible in single statement.

```butter
mut foo = 10;
mut bar = 20;
foo, bar <- bar, foo;
std.assert(foo == 20);
std.assert(bar == 10);
```

We recommend using parallel assignment only when swapping values.
