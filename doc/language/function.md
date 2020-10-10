# Function

Function is a reusable expression that are executed on every call. It is defined by 2 parts, the parameters and the body.

```butter
reverse = arr => {
    reverse = [];
    for elem in arr {
        reverse <- [elem, ..reverse];
    }
    reverse
};
```

## Parameters

The parameter have the same syntax as struct unpacking, except it cannot have rest syntax. It is placed before the `=>`.

It can have simpler syntax, if theres only at most one parameter, you can omit the parentheses.

```butter
report_favorite = (name, favorite) => {
    std::println([..name, .."'s favorite is ", ..favorite, .."!"]);
};
report_niceness = favorite => std::println([..favorite, .." is really nice!"]);
report_end ==> std::println("done!");
```

Notice `==>` is really `=` and `=>`.

## Body

The body is evaluated on every call. It is placed after the `=>`. Body can only contain single expression, so block is used for more complex body.

## Return

Functions can contain return expression, these immediately exits the function body and use its expression as the value of the function call.

```butter
reverse = arr => {
    if arr.len >= 1 {
        return arr;
    }
    reverse = [];
    for elem in arr {
        reverse <- [elem, ..reverse];
    }
    reverse
};
```

## Calling

Calling executes the body of a function. The arguments have similar syntax to struct initialization.

```butter
report_favorite(name: "Alex", favorite: "butter toast");
```

You can omit the names and Butter will use the order of parameter.

```butter
report_favorite("Alex", "butter toast");
```

If there's only one parameter, then you can use any name on argument.

```butter
report_niceness(any_arbitrary_name: "tomato salad");

food = "tomato salad";
report_niceness(food);  -- note that this is actually report_niceness(food: food)
```

Additionally, if you omitted an argument, it will be `null`, make sure the function can handle it.

```butter
say_hello = name => {
    name = name ?? "stranger";
    std::println([.."Hello ", ..name]);
}
say_hello();
```
