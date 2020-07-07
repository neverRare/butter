# Struct

Struct value is a collection of field whose fields are pairs of name and another value. Unlike other languages, you can create struct value on the fly without having to specify its type.

```butter
car = (
    color: "red",
    brand: "a nice brand",
);
```

You can create another struct value based on existing struct value or a reference to it and extend it with more fields. You can splat as many struct as you like as long as every fields have unique name.

```butter
car = (
    ..car,
    price: 100,
);
```

## Field access

A field from struct or a reference to struct can be accessed via `.`.

```butter
color = car.color;
```
