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
{-
    std.print("this code is ignored, nothing would be printed");
-}
```

This can be nested, but the content must be lexable.

## Dict and Set

```butter
-- option 1
map = #(10 = 20, 20 = 40);
set = #[10, 20, 30];

-- option 2
map = dict(10 = 20, 20 = 40);
set = set[10, 20, 30];
```

Access and manipulation? How??

## Type alias

```butter
alias Option(a) = @some a | @none;
```

## Type annotation

For functions

```butter
forall(a, b):
map_option(val: Option(a), mapper: a -> b) -> Option(b) =>
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

There must be first class iterators too.

## Module system

```butter
-- option 1
math = mod (
    pi = 3.14;
    sqrt(num) => {
        -- ...
    }
);

-- option 2
mod math {
    pi = 3.14;
    sqrt(num) => {
        -- ...
    }
}
```

Module in different file.

```butter
-- option 1
math = mod math;

-- option 2
mod math;
```

Importing from nested module, would be similar to declaration.

```butter
pi = math.pi;
-- or with unpacking pattern
(pi,) = math;
```

## Visibility system

```butter
pub greet(name) => "hello " ++ name ++ "!";
```

## Pipeline operator

```butter
"hello world" |> std.print;
```

## Partial application

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
foo = cell 10;
bar = num_a;
```

Casting to reference, `cell_inner` would be a weak keyword.

```butter
mut num = &bar.cell_inner;
num^ <- num^ + 1;
std.assert(foo.cell_inner == 11);
```

## Never

It should never be reachable enforced by refinement type.

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

```butter
if val =: @some val {
    -- ...
} else {
    -- ...
}
```

## Match else

Useful for unwrapping.

```butter
@some val = val else { std.abort() };
```

## Identifier as compile-time value

```butter
map_tagged(val, tag, fn) =>
    match val {
        @$tag val => @$tag fn(val),
        val => val,
    };

map_tagged(val, $some, (val) => val + 3);
```

## Traits

```butter
forall(a):
trait Eq(a) {
    equal(a: &a, b: &b) -> Bool;
}
forall(a):
given Eq(a):
impl Eq([a]) {
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
derive Eq(_):
pub newtype Point(
    pub x: Num,
    pub y: Num,
);
```

They won't have trait implementation by default and each field can have its own visibility (for structural record, every field is public).

Generics? How??
