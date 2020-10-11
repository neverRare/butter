# Null and Optional

Null represents absence of value. This is often used with other types to produce optional types.

```butter
input = {
    input = std::read();
    if input.len > 0 { input } else { null }
};
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

Few operators have counterparts for handling optionals. These are [field access](struct.md#field-access) and array [element access](array.md#element-access) or [slice](array.md#slice). The syntaxes have `?` before the usual operator, like `?.` and `?[...]` respectively.

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
std::assert(cat_name != null);
```
