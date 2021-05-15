# If

If block is used to conditionally execute statements.

```butter
input = std.get_line();
if input.len == 0 {
    std.println("please input something meaningful");
}
```

You can add an `else` branch which gets executed when the condition isn't true

```butter
input = std.get_line();
if input.len == 0 {
    std.println("please input something meaningful");
} else {
    std.println("thanks for the input!");
}
```

And you can branch as many `if`s as necessary.

```butter
password = std.get_line();
len = input.len;
if len == 0 {
    std.println("please input a password");
} else if len < 8 {
    std.println("password too short");
} else {
    std.println("alright!");
}
```

`if` is an expression, just like blocks, it will evaluate to the last expression if its last semicolon is omitted. In this case, there should be a hanging `else`.

```butter
input = std.get_line();
message = if input.len == 0 {
    "please input something meaningful"
} else {
    "thanks for the input!"
};
std.println(message);
```
