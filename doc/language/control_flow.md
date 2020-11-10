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

## For

`for` iterates over an [array].

[array]: array.md

```butter
for fruit in ["apple", "banana", "cherry"] {
    std.println(fruit ++ " is yummy!");
}
```

## While

`while` repeatedly executes the body while the condition is true.

```butter
i = 0;
while i < 10 {
    std.println("hello!");
    i <- i + 1;
}
```

## Loop

`loop` creates an infinite loop.

```butter
loop {
    std.println("this is an infinite loop!");
}
```

## Break

The break expression is used to immediately exit loop statements.

```butter
counter = 0;
while counter < 6 {
    if counter == 3 {
        break;  -- immediately exit the loop despite not being counter < 6
    }
    counter <- counter + 1;
}
std.assert(counter == 3);
```

A break expression can be given an expression, this is only applicable to `loop`. It must be preceded by an equal sign `=` after the `break` keyword and the label if theres any. The loop expression will evaluate to the expression of whichever broke the loop.

```butter
counter = 0;
result = loop {
    counter += 1;
    if counter == 10 {
        break = counter * 2;  -- exit the loop and `result` will be counter * 2
    }
};
std.assert(result == 20);
```

## Continue

The continue expression will stop the current iteration then continues to next iteration.

```butter
filtered_num = [];
for num in [1..10] {
    if num % 2 == 0 {
        continue;
    }
    filtered_num <- [*filtered_num, num];
}
```

## Label

Break and continue is normally associated with the innermost loop containing the expression.

If you wish to associate higher loop, you can use labels. Labels are used to disambiguate nested loops. You can use the following as label.

- Keyword of the loop (`for`, `while`, or `loop`)
- Variable declaration with `loop` as value (the `var` in `var = loop { ... }`)
- Iteration variable of `for` loop (the `i` in `for i in ... { ... }`)

There are cases where above is not enough, Butter doesn't currently have a system for those.

TODO example
