# Struct

Struct value is a collection of field whose fields are pairs of name and another value.

```butter
car = (
    color = "red",
    brand = "a nice brand",
);
```

You can create another struct value based on existing struct value and extend it with more fields. You can splat as many struct as you like as long as every fields have unique name.

```butter
car = (
    *car,
    price = 100,
);
```

You can use field declaration shortcut if the value is stored in a variable with the same name as desired field.

```butter
color = "red";
brand = "a nice brand";
car = (color, brand);
```

Grouping and struct shares the same delimiter `()`. There is a rare case where ambiguity can arise: When defining a struct with single field using the shortcut syntax. Butter considers this as grouping. If struct is expected, you can easily add a trailing comma.

```butter
color = "red";

car = (red);  -- grouping, similar to `red`
car = (red,);  -- struct, similar to `(red: red)`
```

## Field access

A field from struct can be accessed via `.`.

```butter
color = car.color;
```
