# Tag

Tag is used for making tagged variant where we represent values with different possible tags.

You can use tags to represent enumerated values.

```butter
color = "red";
color = match color {
    "red" => @red,
    "yellow" => @yellow,
    "green" => @green,
    "blue" => @blue,
    _ => std.panic("color not in choices"),
};
```

Tags can be associated with a value. This allows us to represent tagged union.

```butter
color = "red";
color = match color {
    "red" => @red,
    "yellow" => @yellow,
    "green" => @green,
    "blue" => @blue,
    color => @rgb color,
};
```

## Pattern matching

We use pattern matching with `match` in order to find out what the value of the tagged union is.

```butter
color = match color {
    @red => "red",
    @yellow => "yellow",
    @green => "green",
    @blue => "blue",
    @rgb color => color,
};
std.print_line("the color is " ++ color);
```
