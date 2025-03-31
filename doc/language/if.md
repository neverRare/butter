# If

If block is used to conditionally execute statements.

```butter
input = get_line();
if input.len == 0 {
    print_line("input cannot be blank");
}
```

You can add an `else` branch which gets executed when the condition isn't `@true`

```butter
input = get_line();
if input.len == 0 {
    print_line("input cannot be blank");
} else {
    print_line("input processed!");
}
```

And you can branch as many `if`s as necessary.

```butter
password = get_line();
len = input.len;
if len == 0 {
    print_line("please input a password");
} else if len < 8 {
    print_line("password too short");
} else {
    print_line("password has changed!");
}
```

`if` is an expression, just like blocks, it will evaluate to the last expression if its last semicolon is omitted. In this case, there should be a hanging `else` (`else` not followed by `if`).

```butter
input = get_line();
message = if input.len == 0 {
    "input cannot be blank"
} else {
    "input processed!"
};
print_line(message);
```

## If match

TODO: explanation

```butter
if val =: @val val {
    -- ...
}
```
