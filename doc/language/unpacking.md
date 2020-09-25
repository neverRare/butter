# Unpacking

Unpacking syntax can be used in variable declaration as well as on function parameter. These are syntactic sugar to accessing field value or element and assigning it to a declared variable. Unpacking can be referred to as destructuring.

## Array unpacking

Array unpacking have similar syntax to array declaration, but they are used on left hand side of declaration, and it does the exact opposite of array declaration.

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

You can unpack a struct via `()`. These assigns field value to a variable with the same name, if you wish to use another variable name, use `:`.

```butter
user = (
    name: "someone",
    email: "someone@example.com",
);
(name: username, email) = user;
```

If you wish to ignore some fields, simply don't write it.

```butter
(email) = user;
```

TODO rest

## Nesting

You can nest unpacking syntax.

```butter
users = [
    (
        name: "someone",
        email: "someone@example.com",
    ),
    (
        name: "anyone",
        email: "anyone@example.com",
    ),
];
[(name, email), .._] = users;
```
