# Unpacking

Unpacking syntax can be used in variable declaration as well as on function parameter. Unpacking can be referred to as destructuring.

## Array unpacking

Array unpacking have similar syntax to array declaration, but they are used on left hand side of declaration, and it does the exact opposite of array declaraion.

```butter
arr = ["hello", "world"];
[first, second] = arr;
std::print([..first, ' ', ..second, '\n']);
```

If you wish to ignore some element, use `_`.

```butter
arr = ["hello", "world"];
[first, _] = arr;
std::print([..first, .." awesome world\n"]);
```

You can unpack from its start or its end then unpack the rest. There can only be at most one rest syntax in array unpacking. This is the counterpart of splat.

```butter
[first, ..rest] = arr;
```

## Struct unpacking

TODO

## Nesting

TODO

## Parameter

TODO
