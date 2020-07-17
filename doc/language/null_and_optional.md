# Null and Optional

Null represents absence of value. This is often used with other types to produce optional types.

```butter
input = {
    input = std::read();
    if input.len > 0 (input) else (null)
};
```

## Handling Null

You can use `== null` or `!= null` to check whether the value is null or not. One of the operand must be exactly `null`.

```butter
name = if input != null (input) else {
    std::println("invalid input");
    abort -1;
};
```

## Null Coalescing Operator

You can use the `??` operator. TODO more explanation.

```butter
name = input ?? "unnamed";
```

## Optional Access

TODO