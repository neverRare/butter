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

If the last statement is an expression and omitted the semicolon after it, the block can be used as expression and it will evaluate to the last expression.

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

`for` iterates over an array.

```butter
for fruit in ["apple", "banana", "cherry"] {
    std::print([..fruit, .." is yummy!"]);
}
```

## While

`while` repeatedly executes the body while the condition is true.

```butter
i = 0;
while i < 10 {
    std::print("hello!\n");
    i <- i + 1;
}
```

## Loop

`loop` creates an infinite loop. It can be exited with `break`.

```butter
loop {
    std::print("this is an infinite loop!\n");
}
```

`loop` is an expression when its `break` have an expression.

```butter
counter = 0;
result = loop {
    counter <- counter + 1;
    if counter == 10 {
        break = counter * 2;
    }
};
std::assert(result == 20);
```

## Break

TODO

## Continue

TODO

## Label

Loop expression such as `loop`, `while`, and `for` and be labelled. This is useful especially when using `break` or `continue` on deep loops.

TODO example
