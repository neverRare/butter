# Ideas

## Unicode support

Currently, string and char literal are just syntactic sugar for array of bytes and bytes encoded in utf8. Unicode support shall be library features:

- Uses UTF-8 encoding.
- There shall be a string type that simply wraps array of numbers.
- Validity of such type shall be enforced by refinement types. Unsure for this, might be a challenge or simply impossible.
- Another type for Unicode bytes and scalar values.

## Raw string

```butter
#"raw strings don't have escape notation"#
```

## Match parameter

```butter
map_option(match val, mapper) => {
    @val val => @val mapper(val),
    @none => @none,
}
```

## Or Pattern

```butter
match num {
    1 | 2 | 3 => ...,
    _ => ...,
}
```

## Range Pattern

```butter
match num {
    1..3 => ...,
    _ => ...,
}
```

## As Var Pattern

```butter
match num {
    1..3 as num => ...,
    _ => ...,
}
-- or
match num {
    num as 1..3 => ...,
    _ => ...,
}
```

## Equal to pattern

Also known as pin pattern or pin operator.

```butter
match foo {
    == bar => std.print_line("it's equal to bar!"),
}
```

## ~~Control flow label~~

This clashes with type annotation syntax. There should not be dedicated syntax for label anyway, it must be syntactically salted.

```butter
outer: while true {
    for i in arr {
        if i == 2 {
            break outer;
        }
    }
}
```

## `upto`

An alternative to control flow label. This will `break` or `continue` the nth closest loop. providing `0` is just the same as not using `upto` at all.

```butter
while true {
    for i in arr {
        if i == 2 {
            break upto 1;
        }
    }
}
```

You may use keyword instead. It won't work when there's multiple loop with the same keyword.

```butter
while true {
    for i in arr {
        if i == 2 {
            break upto while;
        }
    }
}
```

Instead of `upto`, `at` keyword may be used.

## Breakable block

```butter
num = {|
    if foo == 10 {
        break 10;
    }
    20
|};
```

## Multiline comment

```butter
-/
    multiline
    comment
/-
```

It won't be nestable. Intended for textual comments rather than disabling span of codes.

## Ignore codeblock

```butter
{-
    std.print("this code is ignored, nothing would be printed");
-}
```

This can be nested, but the content must be lexable. Intended for disabling span of codes.

## Raw identifier

```butter
`loop` = parser(...);
```

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
alias Option(a) = @val a | @none;
```

## Left to right var declaration

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

## Iterators

Options for design and implementation:

- Have traits for iterators. Simplest implementation but lessens the ergonomics.
- Have iterator as first-class type. This will use dynamic dispatch but ergonomics can be great. A generalization for this approach would be an implementation of dynamic object with certain trait.

## Iterator literal

```butter
iter("apple", "banana", "cherry")
iter(1.<3)
-- or
#("apple", "banana", "cherry")
#(1.<3)
```

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
(= pi) = math;
-- or with left to right declaration
math =: (= pi);
```

## Declaration shorthand

```butter
= math.pi;
-- the same as
pi = math.pi;
```

## Visibility system

```butter
pub greet(name) => "hello " ++ name ++ "!";

-- public to only select module
pub(path.to.module) greet(name) => "hello " ++ name ++ "!";
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

An escape hatch for "no shared mutable" rule. There might be a better keyword other than `cell`.

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

```butter
if val =: @val val {
    -- ...
} else {
    -- ...
}
```

## Match else

Useful for unwrapping.

```butter
@val val = val else { std.panic() };
```

## Identifier as compile-time value

```butter
map_tagged(val, tag, fn) =>match val {
    @$tag val => @$tag fn(val),
    val => val,
}

map_tagged(val, $val, (val) => val + 3);
```

## Traits

```butter
:(a):
trait Eq(a) {
    equal(a: &a, b: &b) -> Bool;
}
:(a):
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
-- declaration
derive Eq(_):
pub newtype Point(
    x: Num,
    y: Num,
);

-- creation
point = Point(x = 10, y = 20);
```

They won't have trait implementation by default and have it's own refined types.

Generics? How??

## Private fields

```butter
-- declaration
derive Eq(_):
newtype Point(
    x: Num,
    #y: Num,
    pub(path.to.module) #z: Num,
);

-- creation
point = Point(x = 10, #y = 20, #z = 30);

-- access
y = point.#y;
```

Anonymous record types have all fields public. Private fields are only applicable for `newtype`. Private fields can have visibility overridden by using `pub`.

## Effect system

```butter
message() => {
    yield item("hello");
    yield item("world");
}
collect_message() => {
    mut arr = [];
    do message() handling {
        item(item) => arr <- >arr ++ [item],
    }
    arr
}
```

This is a very novel feature. Unsure about the implementation, the runtime impact, and many more. Need more study before implementing.
