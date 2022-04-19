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

TODO: use other example

```butter
reverse(arr) => {
    if arr.len >= 1 {
        return arr;
    }
    mut reverse = [];
    for elem in >arr {
        reverse <- [elem] ++ reverse;
    }
    reverse
}
```

## Calling

Calling executes the body of a function. Within the arguments, you can use either record syntax (named arguments) or tuple syntax (unnamed arguments).

```butter
report_favorite(name, favorite) => {
    std.println(name ++ "'s favorite is " ++ favorite ++ "!");
}

-- calling with record syntax
report_favorite(name = "Alex", favorite = "butter toast");
-- calling with tuple syntax
report_favorite("Alex", "butter toast");
```

## Scoping of named function

**Note:** these are not fully implemented yet.

Named function can act like like [variables], although it have special scoping rules.

[variables]: variable_and_assignment.md

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

## Capturing

TODO
