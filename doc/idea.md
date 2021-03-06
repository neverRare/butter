# Ideas

## Equal to pattern

Also known as pin pattern or pin operator.

```butter
match foo {
    == bar => std.println("it's equal to bar!"),
}
```

## Control flow label

This clashes with [type annotation](#type-annotation). There should not be dedicated syntax for label anyway, it must be syntactically salted.

```butter
outer: while true {
    for i in arr {
        if i == 2 {
            break outer;
        }
    }
}
```

## Multiline comment

```butter
-/
    multiline
    comment
/-
```

It won't be nestable.

## Ignore codeblock

```butter
{--
    std.print("this code is ignored, nothing would be printed");
}
```

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
map_option(val: Option('a), mapper: 'a -> 'b) -> Option('b) =>
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

Records could double as namespace.

```butter
math = (
    pi = 3.14,
    sqrt(num) => {
        -- ...
    }
);
```

Maybe a record something like rust's module.

```butter
math = mod math;
```

Importing from modules could be the same as declaration.

```butter
pi = math.pi;
```

## Visibility system

```butter
pub greet(name) => "hello " ++ name ++ "!";
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

## Shareable mutable container

An escape hatch for ownership and no mutable alias rule.

```butter
cell 10
```

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

## If match, while match

Similar to `if let` and `while let` on rust.

## Match else

Useful for unwrapping.

```butter
@some val = val else { std.abort() };
```

## Polymorphism with tag and field name

```butter
map_tagged(val, tag = @$tag, fn) =>
    match val {
        @$tag val => @$tag fn(val),
        val => val,
    };
```

## Traits

```butter
trait Eq('val) {
    equal(a: &'val, b: &'val) -> Bool;
}
given Eq('val)
impl Eq(['val]) {
    equal(a, b) => {
        if a^.len != b^.len { return false; }
        for i in [0.< a^.len] {
            if a^[i] != b^[i] { return false; }
        }
        true
    }
}
```

## New nominal type

```butter
newtype Point(a: Num, b: Num);
```

They won't have trait implementation by default and each field can have its own visibility (for structural record, every field is public).
