# Lifetime

Lifetime defines the span of code where the place have value in it ([initialization]) and it ends when the place is last used.

[initialization]: move_and_initialization.md#initialization

```butter
message = "hello world";  -- here, the lifetime of `message` start
                          -- as it is declared and initialized with a value
std.print_line(message);  -- here, the lifetime of `message` end where it is last used
```

Lifetimes can be broken when move and assignment is used

```butter
mut message = "hello world";  -- lifetime of `message` starts here
foo = >message;               -- the lifetime ends here
                              -- `message` can't be used here
message <- "hi world";        -- `message` starts here again
std.print_line(message);      -- here, the lifetime of `message` end
```

Note that lifetimes aren't linear as there are control flows that can change the flow of execution and hence lifetimes can diverge and converge.

TODO: lifetimes can be granular over record and tuples, maybe an explanation for those
