# Variable Declaration

Variables are declared via `=`.

```butter
foo = 10;
bar = @false;
```

Variables can shadow previously declared variable with the same name, either on the same or on upper scope.

```butter
foo = 10;
{
    foo = @false;
    assert(foo == @false);
}
assert(foo == 10);
foo = 20;
assert(foo == 20);
foo = foo == 20;
assert(foo == @true);
```

## Left to right var declaration

TODO: explanation

```butter
num = 10;
10 =: num;
```

Unpacking complex values

```butter
account =: (
    = name,
    = email,
    birth_date = (
        = day,
        = month,
        = year,
    ),
);
```

## Declaration shorthand

TODO: explanation

```butter
= math.pi;
-- the same as
pi = math.pi;
```

Import many.

```
= math.(pi, sqrt);
```

## Match else

Useful for unwrapping.

```butter
@val val = val else { panic() };
```
