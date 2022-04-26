# Match

`match` expression matches a value against into a line of pattern until it matches. Once matched, the expression next to the matched pattern will be evaluated and used as the value of the `match` expression.

```butter
color = @rgb (15, 120, 211);
value = match color {
    @rgb (red, green, blue) => min([red, green, blue]),
    @hsv (_, _, value) => value,
};
```

`match` must be exhaustive, meaning it handles all of the possible patterns of the value.
