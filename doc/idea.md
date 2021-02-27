# Ideas

## Map and Set

```butter
map = #(10 = 20, 20 = 40);
set = #[10, 20, 30];
```

Sometimes called *hash*map and *hash*set ;). I still have no idea on manipulations however.

## Tagged values

```butter
-- no associated value
@tag_name
@tag_name()

-- single associated value
@tag_name expr
@tag_name(expr)

-- struct as associated value
@tag_name(name1 = expr1, name2 = expr2)

-- array as associated value
@tag_name[expr1, expr2]
```

This could be structurally typed like in TypeScript's fake discriminated union but with better syntax like in Rust. This could also replace the current null system.

## Pattern and match

```butter
-- matching tagged value
map_option(val, mapper) => match val {
    @some val => @some mapper(val),
    @none => @none,
};

-- matching array
map_array(arr, mapper) => match arr {
    [] => [],
    [item, *rest] => [mapper(item)] ++ map_array(arr = rest, mapper),
};
```

## Type alias

`<A>` here are type variables. Syntax' ugly at the moment, it'll change.

```butter
Option :: <A> => @some(<A>) | @none;
```

## Item type annotation

```butter
:: (val = Option(<A>), mapper = <A> => <B>) => Option(<B>);
map_option(val, mapper) => match val {
    @some(val) => @some(mapper(val)),
    @none => @none,
};

:: (val = [<A>; <L>], mapper = <A> => <B>) => [<B>; <L>];
map_array(arr, mapper) => match arr {
    [] => [],
    [item, *rest] => [mapper(item)] ++ map_array(arr = rest, mapper),
};
```

## Value type annotation

```butter
foo = [] as [Num; 0];
```

## Left to right var declaration

```butter
num = 10;
10 =: num;

-- unpacking complex values
account =: (
    name,
    email,
    birth_date = (
        month,
        day,
        year,
    ),
);
```

## Array comprehension

```butter
numbers = [1..100];

-- python-like syntax
doubles = [n * 2 for n in numbers];
no_doubles = [n for n in numbers if n % 2 == 0];

-- alternative syntax
doubles = [n * 2 : n in numbers];
no_doubles = [n : n in numbers : n % 2 == 0];
```

## Module system

Structs could double as namespace.

```butter
math = (
    pi = 3.14,
    sqrt(num) => {
        -- ...
    }
);
```

Maybe a struct something like rust's module.

```butter
math = mod math;
```

Importing from modules could be the same as declaration.

```butter
pi = math.pi;
```

## Function as operator

```butter
and(a, b) => match (a, b) {
    (a = @true, b = @true) => @true,
    _ => @false,
};
not(a) => match a {
    @true => @false,
    @false => @true,
};
@true `and` @false;  -- @false
`not` @true;  -- also @false
```
