# Ideas

## Unicode support

Currently, string and char literal are just syntactic sugar for array of bytes and bytes encoded in utf8. Unicode support shall be library features:

- Uses UTF-8 encoding.
- There shall be a string type that simply wraps array of numbers.
- Validity of such type shall be enforced by refinement types. Unsure for this, might be a challenge or simply impossible.
- Another type for Unicode bytes and scalar values.

## String interpolation

Doesn't evaluates to string but maybe to something like `Iter(Str)` for efficiency.

```butter
print_line(f"value is \(value)");
```

Debug printing, pretty printing, padding, alignment, etc. shall use normal functions instead of special syntaxes.

Debug printing and pretty printing

```butter
print_line(f"value is \(debug(value))");
print_line(f"value is \(pretty_print(value))");
```

Padding, alignment etc.

```butter
name = pad_end(name, 20);
value = pad_start(value, 5);
print_line(f"\(name) \(value)");
```

Multiline strings are automatically dedented.

## Match parameter

```butter
map_option(match val, mapper) => {
    @val val => @val mapper(val),
    @none => @none,
}
```

## `upto`

An alternative to control flow label. This will `break` or `continue` the nth closest loop. providing `0` is just the same as not using `upto` at all.

```butter
while @true {
    for i in arr {
        if i == 2 {
            break upto 1;
        }
    }
}
```

You may use keyword instead. It won't work when there's multiple loop with the same keyword.

```butter
while @true {
    for i in arr {
        if i == 2 {
            break upto while;
        }
    }
}
```

Instead of `upto`, `at` keyword may be used.

## Dict and Set

Will be provided by std library.

## Type alias

```butter
alias Option(a) = @val a | @none;
```

## Iterators

Iterators will use [existential types](#existential-types).

```butter
:(a):
trait Iterator(a) {
    alias Item;
    next(self : &:mut a) -> Item;
}
alias Iter(a) = impl(b) b where Iterator(b).Item = a;
```

## Iterator literal

```butter
#["apple", "banana", "cherry"]
#[1 .< 3]
```

Iterator comprehension

```butter
#[val; for val in array]

-- if guards
#[val; for val in array; if val > 0]

-- skip when pattern matching fails
#[val; for @val val in array]
```

## Module system

```butter
mod math {
    share pi = 3.14;
    sqrt(num) => {
        -- ...
    }
}
```

Module in different file.

```butter
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

Special names

- `self`
- `super`
- `lib`
- `global`

## Bind everything pattern

Only usable for modules, not records.

```butter
(*) = math;
```

With declaration shorthand.

```butter
= math.*;
```

## Visibility system

```butter
pub greet(name) => "hello " ++ name ++ "!";

-- public to only select module
pub(path.to.module) greet(name) => "hello " ++ name ++ "!";
```

## Pipeline operator

```butter
"hello world" |> print_line;
```

## Partial application

```butter
add(a, b) => a + b;

result = 40 |> add(?, 2);
```

## Shareable mutable container

Provided by std library.

```butter
share foo = cell(10);
share bar = foo;
```

```butter
mut num = extract_cell(&foo);
num^ <- num^ + 1;
assert(extract_cell(&foo) == 11);
```

## Never

It should never be reachable enforced by refinement type. Provided by std library.

```butter
-- this could be in std
expect(condition) => if condition {} else { never() };

prime_factor(num) => {
    expect(num % 1 == 0);
    expect(num >= 1);
    if num == 1 {
        []
    } else {
        for i in #[2 .. num] {
            if num % i == 0 {
                return [i] ++ prime_factor(num / i);
            }
        }
        never()
    }
}
```

## Compile-time variable holding identifier

```butter
map_tagged(val, tag, fn) => match val {
    @$tag val => @$tag fn(val),
    val => val,
}

map_tagged(val, "val", (val) => val + 3);
```

## Traits

```butter
:(a):
trait Eq(a) {
    equal(left : &a, right : &b) -> Bool;
}
:(a):
where Eq(a):
impl Eq([a]) {
    equal(left, right) => {
        if left^.len /= right^.len { return @false; }
        for i in #[0 .< left^.len] {
            if left^[i] /= right^[i] { return @false; }
        }
        @true
    }
}
```

Implementing traits on tuple

```butter
impl Eq(()) {
    equal(left, right) => true;
}

:(a, rest):
where Eq(a):
where Eq(rest):
impl Eq((a, *rest)) {
    equal(left, right) => {
        &>(left, *left_rest) = left;
        &>(right, *right_rest) = right;
        left == right && left_rest == right_rest;
    }
}
```

Implementing traits on records with meta-programming using [compile-time variable holding identifier](#compile-time-variable-holding-identifier)

TODO: ordering of fields, HOW?

```butter
impl Eq(()) {
    equal(left, right) => true;
}

:(name, a, rest):
where Eq(a):
where Eq(rest):
impl Eq(($name : a, *rest)) {
    equal(left, right) => {
        &>($name = left, *left_rest) = left;
        &>($name = right, *right_rest) = right;
        left == right_rest && left_rest == right_rest;
    }
}
```

On tagged union, again with meta-programming.

```butter
impl Eq(Never) {
    equal(left, right) => never();
}

:(tag, a, rest):
where Eq(a):
where Eq(rest):
impl Eq(@$tag a | rest) {
    equal(left, right) => {
        match (left, right) {
            (&>@$tag left, &>@$tag right) => left == right;
            (&>@$tag _, _) | (_, &>@$tag _) => @false;
            (left, right) => left == right;
        }
    }
}
```

## Existential types

```butter
value : impl Eq;
```

More complex syntax:

```butter
value : impl(a) a where Eq(a);
```

## New nominal type

```butter
-- declaration
pub newtype Point(
    x : Num,
    y : Num,
);

-- creation
point = Point(x = 10, y = 20);
```

It won't have trait implementation by default and have it's own refined types.

Generics:

```butter
:(a):
pub newtype Extended(@neg_inf | @fin a | @inf);

-- or

pub newtype Extended:(a)(@neg_inf | @fin a | @inf);
```

To achieve nominal typing, instances are simply regular record or tuple but it holds a value on the special field `__nominal`. The type of this value is also a special type acting like a unit but has nominal typing. This special type is accessible from `Type.NominalUnit` where `Type` is the name of the type.

```butter
point = Point(x = 10, y = 20);

-- is similar to

point = (__nominal: Point.NominalUnit(), x = 10, y = 20);
```

The special field and type should only be relevant when implementing auto trait implementation.

`__nominal` is not a typical field. Tuples can have it. `__nominal` is a keyword and is different from `n"__nominal"`.

## Auto-implement traits

Usually, this will rely on trait implementation on regular tuples and records, making use of the special nominal field if necessary.

```butter
auto Eq(a) {
    impl Eq(a.NominalUnit) {
        equal(left, right) => true;
    }
}
```

```butter
impl auto Eq(Point);
```

## Private fields

```butter
-- declaration
newtype Point(
    x : Num,
    #y : Num,
    pub(path.to.module) #z : Num,
);

-- creation
point = Point(x = 10, #y = 20, #z = 30);

-- access
y = point.#y;

-- pattern matching will require explicit record type name
Point(= #y) = point;

-- this is an error
(= #y) = point;
```

Anonymous record types have all fields public. Private fields are only applicable for `newtype`. Private fields can have visibility overridden by using `pub`.
