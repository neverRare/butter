# Clone

Clone creates a deep-copy of a value. Any mutation within the original value will not affect the copied value and vice-versa.

```butter
foo = 10;
bar = clone foo;
foo <- 20;
std::assert(bar == 10);
```

Additionally, you can't clone a function.

## Implicit copy

Few types have implicit copy, especially scalar ones, this includes [number] and [boolean], whether [optional] or not. You can avoid this by using [references].

[number]: number.md
[boolean]: boolean.md
[optional]: null_and_optional.md
[references]: reference.md

The above example can be simplified as shown below as numbers can be implicitly copied.

```butter
foo = 10;
bar = foo;
foo <- 20;
std::assert(bar == 10);
```
