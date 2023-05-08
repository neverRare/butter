# Type annotation

**Note:** these are not implemented yet

While Butter can infer most types of such value according to how it is used, sometimes it can fail and we need to explicitly annotate it. It's also sometimes good to explicitly annotate types, Butter can wrongly infer types when there's a mistake on the code, this can happen on huge codes.

Type annotation are used to explicitly define the type of expression, pattern, and function parameter and return type, also known as function signature.

For type annotation of expressions and patterns, we use `:`.

```butter
val : Num = 10;

-- you can directly annotate expression as well
val = 10 : Num;
```

## Function signature and generics

For function signature, we use `:` for parameters and `->` for return type.

```butter
say_hello(name : Str) -> Str => "hello " ++ name ++ "!";
```

We can define generics by using `:():`.

```butter
:(a):
concatenate(left : [a], right : [a]) -> [a] => left ++ right;
```

TODO: explain what generics do

## Wildcard

You want type annotation but you don't want full type annotation on a single value, this is where wildcard type can help. This is represented by `_`. This let Butter infer it.

```butter
-- `arr` is explicitly an array but the type of the element is left inferred
arr : [_] = [];
```

## Predefined types

Numbers have type `Num` and booleans have type `Bool`. There is also `Char` and `Str` which are simply `Num` and `[Num]` respectively

```butter
num : Num = 10;
truth : Bool = true;
char : Char = 'a';
string : Str = "Hello World";
```

## Array types

Array types are expressed as `[ty]` where `ty` is the type of the element.

```butter
fruits : [Str] = ["apple", "banana", "cherry"];
```

## Record types

TODO: explanation

```butter
name(user : &{name : Str, *_}) -> Str {
    &user^.name
}
```

## Tuple types

TODO: explanation

```butter
:(a)
first(tuple : &(a, *_)) -> a {
    &tuple^.0
}
```

## Tagged union types

TODO: explanation and better example

```butter
val : (@val _, @none)

-- row
val : (@val _, *_)
```

## Reference types

TODO: explanation and better example

```butter
val : &:mut_var /l_var ty
```

## Function types

TODO
