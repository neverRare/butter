# Ideas

## Map and Set

```butter
map = #(10 = 20, 20 = 40);
set = #[10, 20, 30];
```

## Type alias

`'a` here are type variables.

```butter
alias Option('a) = @some 'a | @none;
```

## Type annotation

For functions

```butter
map_option(val: Option('a), mapper: 'a => 'b) => : Option('b):
    match val {
        @some(val) => @some(mapper(val)),
        @none => @none,
    };
```

For variables

```butter
foo: [Num] = [];
```

For expressions

```butter
foo = []: [Num];
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

## Chain operator

```butter
"hello world" |> std.print;
```

## Function Currying

```butter
add(a, b) => a + b;

result = 40 |> add(?, 2);
```

## Uninitialized value

```butter
foo = undef;
foo <- 10;
```

## Interiorly mutable reference

```butter
&cell 10
```

Should be behind a reference to make it clear that the content is never implicitly copied.

## Never

It should never be reachable, it is guaranteed by refinement type (or not).

```butter
-- this could be in std
expect(condition) => if condition {} else { never };

prime_factor(num) => {
    expect(num % 1 == 0);
    expect(num >= 1);
    if num == 1 {
        []
    } else {
        for i in [2..num] {
            if num % i == 0 {
                return [i] ++ prime_factor(num / i);
            }
        }
        never
    }
}
```
