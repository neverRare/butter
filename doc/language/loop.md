# Loop control flow

Butter have various kinds of expression for loops.

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
mut counter = 0;
while counter < 10 {
    std.println("hello!");
    counter <- counter + 1;
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
mut counter = 0;
while counter < 6 {
    if counter == 3 {
        break;  -- immediately exit the loop despite not being counter < 6
    }
    counter <- counter + 1;
}
std.assert(counter == 3);
```

A break expression can be given an expression, this is only applicable to `loop`. The loop expression will evaluate to the expression of whichever broke the loop.

```butter
mut counter = 0;
result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;  -- exit the loop and `result` will be counter * 2
    }
};
std.assert(result == 20);
```

## Continue

The continue expression will stop the current iteration then continues to next iteration.

```butter
mut filtered_num = [];
for num in [1..10] {
    if num % 2 == 0 {
        continue;
    }
    filtered_num <- filtered_num ++ [num];
}
```
