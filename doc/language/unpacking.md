# Unpacking

Unpacking syntax can be used in variable declaration as well as on function parameter. These are syntactic sugar to accessing field value or element and assigning it to a declared variable. Unpacking can be referred to as destructuring.

## Array

Array unpacking have similar syntax to array declaration, but they are used on left hand side of declaration, and it does the exact opposite of array declaration.

```butter
arr = ["hello", "world"];
[first, second] = arr;
std.println(first ++ " " ++ second);
```

If you wish to ignore some element, use `_`.

```butter
arr = ["hello", "world"];
[first, _] = arr;
std.println(first ++ " awesome world");
```

You can unpack from its start or its end then unpack the rest as an another array. There can only be at most one rest syntax in array unpacking. This is the counterpart of splat.

```butter
[first, *rest] = arr;
```

## Struct

You can unpack a struct via `()`. These assigns field value to a variable with the same name. If you wish to use another variable name, you can use `=`.

```butter
user = (
    name = "someone",
    email = "someone@example.com",
);
(name = username, email) = user;
```

If you wish to ignore some fields, either don't write it or rename it to `_`.

```butter
(email) = user;
-- or
(name = _, email) = user;
```

You can partially unpack fields and unpack the rest to another struct. There can be only at most one rest syntax in struct unpacking.

```butter
car = (
    color = "red",
    brand = "a nice brand",
    price = 100,
);
(price, *car) = car;
```

## Nesting

You can nest unpacking syntax.

```butter
users = [
    (
        name = "someone",
        email = "someone@example.com",
    ),
    (
        name = "anyone",
        email = "anyone@example.com",
    ),
];
[(name, email), *_] = users;
```
