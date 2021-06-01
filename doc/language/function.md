# Function

Function is a reusable expression that are executed on every call.

```butter
say_hello(name) => "hello " ++ name ++ "!";
```

## Unnamed function

The example above defines named function. Butter also have unnamed functions where we can treat it like a value, it can be used as expression.

```butter
compose(f, g) => (x) => f(g(x));
```

## Parameters

The parameter have the same syntax as [struct unpacking], except it cannot have rest syntax.

[struct unpacking]: unpacking.md#struct

Unnamed functions can have simplified parameter syntax when it have at most 1 parameter: the parentheses can be omitted. Unnamed functions with 1 parameter would look like `param => body` and those with no parameter would simply be `=> body`.

The example above can be rewritten to

```butter
compose(f, g) => x => f(g(x));
```

## Body

The body is evaluated on every call. Body can only contain single expression, so [block] is used for more complex body.

[block]: control_flow.md#block

## Return

Return expressions immediately exits the function body and use its expression as the value of the function call.

TODO: use other example

```butter
reverse(arr) => {
    if arr.len >= 1 {
        return arr;
    }
    mut reverse = [];
    for elem in arr {
        reverse <- [elem] ++ reverse;
    }
    reverse
}
```

## Calling

Calling executes the body of a function. The arguments have similar syntax to struct initialization.

```butter
report_favorite(name, favorite) => {
    std.println(name ++ "'s favorite is " ++ favorite ++ "!");
}
report_favorite(name = "Alex", favorite = "butter toast");
```

You can omit the names and Butter will use the order of parameter.

```butter
report_favorite("Alex", "butter toast");
```

If there's only one parameter, then you can use any name on argument.

```butter
report_niceness(something) => std.println(something ++ " is really nice!");

report_niceness(any_arbitrary_name = "tomato salad");

food = "tomato salad";
report_niceness(food);  -- note that this is actually report_niceness(food = food)
```

## Scoping of named function

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
        abort;
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
