# Record

Record is an association of names to values with variety of types. These pairs are called field or record field.

```butter
car = (
    color = "red",
    brand = "a nice brand",
);
```

You can create another record value based on existing record value and extend it with more fields. Every fields must have unique name.

```butter
car = (
    *car,
    price = 100,
);
```

You can use field punning syntax if the value is stored in a variable with the same name as desired field.

```butter
color = "red";
brand = "a nice brand";
car = (= color, = brand);
```

## Field access

A field from record can be accessed via `.`.

```butter
color = car.color;
```
