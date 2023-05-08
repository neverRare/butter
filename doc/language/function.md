# Function

Function is a reusable set of code that are executed on every call.

```butter
say_hello(name) => "hello " ++ name ++ "!";
```

## Unnamed function

The example above defines named function. Butter also have unnamed functions where we can treat it like a value, it can be used as expression.

```butter
compose(f, g) => (x) => f(g(x));
```

## Return

Return expressions immediately exits the function body and use its expression as the value of the function call.

```butter
factorial(num) => {
    if num == 0 {
        return 1;
    }
    mut result = 1;
    for i in [1..num] {
        result <- result * i;
    }
    result
}
```

## Calling

Calling executes the body of a function. Within the arguments, you can use either name the arguments or left them unnamed. We cannot have mix of named and unnamed arguments.

```butter
report_favorite(name, favorite) => {
    std.print_line(name ++ "'s favorite is " ++ favorite ++ "!");
}

-- calling with named arguments
report_favorite(name = "Alex", favorite = "butter toast");
-- calling with unnamed arguments
report_favorite("Alex", "butter toast");
```

## Scoping of named function

**Note:** these are not fully implemented yet.

Named function can act like variables from [variable declaration], although it have special scoping rules.

[variable declaration]: ./variable_declaration.md

First, there must be no other named function nor variable with the same name on the same scope. This is unlike variable where shadowing will be performed.

```butter
is_even(value) => value % 2 == 0;

is_even = true;  -- error
```

Second, unless shadowed, the body can access the function itself. With this, recursion can be achieved.

```butter
fibonacci(nth) => {
    if nth < 0 {
        std.panic()
    } else if nth <= 1 {
        nth
    } else {
        fibonacci(nth - 1) + fibonacci(nth - 2)
    }
}
```

Third, unless shadowed, the variable is accessible in places before it is declared.

```butter
foo = 10;
increment(&foo);
std.assert(foo == 11);

increment(num) => num <- num + 1;
```

With this, you can implement mutually recursive functions. You can't, however, make mutual recursion between variable declaration and function.

```butter
is_even(num) => {
    match num {
        0 => true,
        num => is_odd(num - 1),
    }
}
is_odd(num) => {
    match num {
        0 => false,
        num => is_even(num - 1),
    }
}
```

## Capturing

Functions can access values outside it. This is known as capturing. Named and unnamed functions differs on how they capture values.

Named functions can only capture values that are [shared].

[shared]: ./share.md

```butter
pi = 3.14;

area_of(circle) => circle^.radius * circle^.radius * pi;
```

TODO: capturing of unnamed function
