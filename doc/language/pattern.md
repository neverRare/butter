# Pattern

Pattern can be used in [variable declaration], [function parameter], and iteration variable in [`for`] loop. These are used to match against the structure of the value and possibly bind a value to a variable.

[variable declaration]: variable_and_assignment.md#declaration
[function parameter]: function.md#parameters
[`for`]: control_flow.md#for

## Ignore

You can discards the value regardless of its type or structure with `_`.

```butter
_ = 10;
```

## Variable

You can bind a value in a variable.

```butter
num = 10;
```

You can mark it as mutable.

```butter
mut num = 10;
num <- 20;
```

You can bind it to a reference.

```butter
ref num = 10;
-- num is a reference to 10
```

You can do both

```butter
ref mut num = 10;
num^ <- 20;
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

## Void

This pattern matches against `void`, nothing special.

## Array

[Array] pattern matches against the length and each element of the array.

[Array]: array.md

```butter
arr = ["hello", "world"];
[first, second] = arr;
std.println(first ++ " " ++ second);
```

You can match against its start or its end then match against the rest as an another array. There can only be at most one rest pattern.

```butter
[first, *rest] = arr;
```

## Struct

[struct] pattern matches against struct. You may use a shortcut syntax where `var` or `ref var` is written instead of `var = var` or `var = ref var` respectively.

[struct]: struct.md

```butter
user = (
    name = "someone",
    email = "someone@example.com",
);
(name = username, email) = user;
```

You can partially match against fields and match against the rest as another struct. There can be only at most one rest pattern.

```butter
car = (
    color = "red",
    brand = "a nice brand",
    price = 100,
);
(price, *car) = car;
```

## Tagged pattern

Tagged pattern matches against the tag and the associated value of a value.

```butter
num = @some 10;
@some num = num;
std.assert(num == 10);
```

## Reference

TODO

`&more_pattern`

## Refutability

TODO: explain the word refutability

These are the patterns that are irrefutable unless they contain refutable pattern.

- Ignore
- Variable
- Struct
- Reference

For tagged pattern, it is refutable according to its tag. Such pattern will only match to values with the same tag.

For array, it is refutable according to its length. `[first, second]` only matches to array with length of 2. For array pattern with rest, it will only match arrays with length greater than or equal to the number of non-rest element patterns. For example, `[first, second, *rest]` will only match to arrays with length greater than or equal to 2.

One exception is if the array only contains rest pattern, it would be irrefutable. However, this isn't really useful as `[*rest]` is just similar to `rest`.

TODO: explain its uses

TODO: explain the cases where a single refutable pattern is already exhaustive enough
