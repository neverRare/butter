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
    std::assert(foo == false);
}
std::assert(foo == 10);
foo = 20;
std::assert(foo == 20);
foo = foo == 20;
std::assert(foo == true);
```

## Assignment

Variables are reassigned via `<-`.

```butter
foo = 10;
foo <- 20;
```

Reassign a value of field of a [struct].

[struct]: struct.md

```butter
foo = (
    prop: 10,
);
foo.prop <- 20;
```

Reassign an element of an [array].

[array]: array.md

```butter
foo = [10];
foo[0] <- 20;
```

Reassign where the [reference] refer to.

[reference]: reference.md

```butter
foo = 10;
bar = 20;
baz = &foo;
&baz <- &bar;
```

## Parallel assignment

You can condense multiple assignment into single statement with parallel assignment. Parallel assignment will also simultaneously perform assignment, making value swapping possible in single statement.

```butter
foo = 10;
bar = 20;
foo, bar <- bar, foo;
std::assert(foo == 20);
std::assert(bar == 10);
```
