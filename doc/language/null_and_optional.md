# Null and Optional

Null represents absence of value. This is often used with other types to produce optional types.

```butter
input = {
    input = std::read();
    if input.len > 0 { input } else { null }
};
```

## Checked operators

Few operators can cause errors, these are `/`, `//`, `%`, for division/modulo to 0 and element access or slice `[...]` for out-of-bound error.

These errors are checked at compile-time, but you can choose to handle errors at runtime with its checked counterpart, which are `/?`, `//?`, `%?`, and `[...?]`. If errors do happen, the result will be null instead.

```butter
num = 10 /? 0;
std::assert(num == null);
```

## Handling Null

You can use `== null` or `!= null` to check whether the value is null or not.

```butter
name = if input != null { input } else {
    std::println("invalid input");
    abort;
};
```

## Null Coalescing Operator

You can use the `??` operator. It evaluates to left expression if its not null, otherwise, the right expression. This short-circuits, which means the right expression only get evaluated when the left is null.

```butter
name = input ?? "unnamed";
```

## Optional Operator

Few operators have counterparts for handling optionals. These are field access and array element access. The syntaxes have `?` before the usual operator, like `?.` and `?[...]` respectively.

These operators short-circuits, if the left expression is null, the whole expression will be evaluated to null without evaluating the right expression and without doing the actual operation.

```butter
adventurer = (
    name: "Alice",
    cat: (
        name: "Dinah",
    ),
    dog: null,
);
dog_name = adventurer.dog?.name;
cat_name = adventurer.cat?.name;
std::assert(dog_name == null);
std::assert(cat_name == null);
```
