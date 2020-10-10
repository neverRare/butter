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

Reassign a value of field.

```butter
foo = (
    prop: 10,
);
foo.prop <- 20;
```

Reassign an element.

```butter
foo = [10];
foo[0] <- 20;
```

Reassign where the reference refer to.

```butter
foo = 10;
bar = 20;
baz = &foo;
&baz <- &bar;
```

## Parallel assignment

TODO explanation

```butter
foo = 10;
bar = 20;
foo, bar <- bar, foo;
std::assert(foo == 20);
std::assert(bar == 10);
```

## Static variables

There is a special kind of variable called static variables. These have special scoping: it is also accessible in places before it is declared.

TODO redefine: non-capturing function should be only static, like javascript's hoisting

```butter
foo = max(10, 20);  -- `max` can be used from here because it is static
max = (a, b) => if a > b { a } else { b };
```

TODO explain how static variable is made
