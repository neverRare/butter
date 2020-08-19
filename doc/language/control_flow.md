# Control flow

Stick have various expression for control flow.

## Block

Block groups statement and creates scope for declaration.

```butter
{
    foo = "hello world";
    len = foo.len;
}
-- foo and len can't be used from here
```

If the last statement is an expression and omitted the semicolon after it, the block can be used as expression and it will evaluate to the last expression. Butter doesn't have uninitialized variable for delayed initialization. Although, it could be simulated with null, using blocks can do better job.

```butter
len = {
    foo = "hello world";
    foo.len
};
```

## If

If block is used to conditionally execute statements.

```butter
input = std::get_line();
if input.len == 0 {
    std::print("please input something meaningful\n");
}
```

You can add an `else` branch which gets executed when the condition isn't true

```butter
input = std::get_line();
if input.len == 0 {
    std::print("please input something meaningful\n");
} else {
    std::print("thanks for the input!\n");
}
```

And you can branch as many `if`s as necessary.

```butter
password = std::get_line();
len = input.len;
if len == 0 {
    std::print("please input a password\n");
} else if len < 8 {
    std::print("password too short\n");
} else {
    std::print("alright!\n");
}
```

`if` is an expression, just like blocks, it will evaluate to the last expression if its last semicolon is omitted. In this case, there should be a hanging `else`.

```butter
input = std::get_line();
message = if input.len == 0 {
    "please input something meaningful\n"
} else {
    "thanks for the input!\n"
};
std::print(message);
```

## For

TODO

## While

TODO

## Loop

TODO
