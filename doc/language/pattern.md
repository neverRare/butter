# Pattern

Pattern can be used in [variable declaration], [match expression], and [`for`] loop. These are used to match against the structure of the value and possibly bind a value to a variable.

[variable declaration]: ./variable_declaration.md
[match expression]: ./match.md
[`for`]: ./control_flow.md#for

## Pattern group

You can wrap pattern inside parentheses `()`. This isn't very useful however as patterns don't have infix operator to disambiguate the precedence.

```butter
(num) = 10;
```

## Literal

You can match against either boolean literal or numeric literal.

```butter
a = 20;

-- pattern match booleans
remarks = match a > 10 {
    true => "it's greater than 10",
    false => "it's not greater than 10",
};
std.print(remarks);

-- pattern match numbers
remarks = match a {
    1 => "it is 1",
    2 => "it is 2",
    _ => "it is something else",
};
std.print(remarks);
```

## Wildcard

You can discards the value regardless of its type or structure with `_`.

```butter
-- discard the whole value
_ = 10;

-- bind the first element of the tuple but discard the rest
pair = (10, "apples");
(count, *_) = pair;
```

## Variable

You can bind a value in a variable.

```butter
num = 10;
```

You can mark it as mutable with `mut`.

```butter
mut num = 10;
num <- 20;
```

You can bind it to a reference with `ref`. This is useful for rebinding references.

```butter
get_name(user) => {
    &(ref name, *_) = user;
    name
}
```

You can do both.

```butter
rename(user, new_name) => {
    &(ref name, *_) = user;
    name^ <- >new_name;
}
```

## Array

[Array] pattern matches against the length and each element of the array.

[array]: array.md

```butter
unwrap_pair(arr) => {
    match arr {
        -- this matches arrays with 2 elements
        [first, second] => (first, second),
        _ => std.panic("passed non-singleton array"),
    }
}
```

You can match against its start or its end then match against the rest as an another array. There can only be at most one rest pattern.

```butter
sum(arr) => {
    match arr {
        -- this matches empty arrays
        [] => 0,
        -- this matches arrays with at least 1 elements
        [first, *rest] => first + sum(arr),
    }
}
```

## Record

[record] pattern matches against record. You may use a field punning syntax where `= var` is written instead of `var = var`.

[record]: ./record.md

```butter
user = (
    name = "someone",
    email = "someone@example.com",
);
(name = username, = email) = user;
```

You can partially match against fields and match against the rest as another record. There can be only at most one rest pattern.

```butter
car = (
    color = "red",
    brand = "a nice brand",
    price = 100,
);
(= price, *car) = car;
```

## Tuple pattern

You can match against elements of the tuple.

```butter
pair = (10, apples);
(how_many, what) = pair;
```

You can partially match against fields from the start or the end of the tuple and match against the rest as another tuple. There can be only at more one rest pattern.

```butter
triple = (10, "green", apples);
(how_many, *pair) = triple;
```

## Tagged pattern

Tagged pattern matches against the tag and the associated value.

```butter
color = @rgb (15, 120, 211);
value = match color {
    @rgb (red, green, blue) => min([red, green, blue]),
    @hsv (_, _, value) => value,
};
```

## Reference

Reference pattern matches against references. These dereferences the value and you may want to rebind it as reference again with `ref`.

```butter
deref(val) => {
    &val = val;
    val;
}
```

## Refutability

TODO: explain the word refutability

These are the patterns that are irrefutable unless they contain refutable pattern.

- Ignore
- Variable
- Record
- Reference

For tagged pattern, it is refutable according to its tag. Such pattern will only match to values with the same tag.

For array, it is refutable according to its length. `[first, second]` only matches to array with length of 2. For array pattern with rest, it will only match arrays with length greater than or equal to the number of non-rest element patterns. For example, `[first, second, *rest]` will only match to arrays with length greater than or equal to 2.

One exception is if the array only contains rest pattern, it would be irrefutable. However, this isn't really useful as `[*rest]` is just similar to `rest`.

TODO: explain its uses

TODO: explain the cases where a single refutable pattern is already exhaustive enough
