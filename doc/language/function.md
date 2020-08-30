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

The parameter have the same syntax as struct unpacking, except it cannot have rest syntax. It can have simpler syntax, if theres only at most one parameter, you can omit the parentheses.

```butter
report_favorite = (name, favorite) => {
    std::print([..name, .."'s favorite is ", ..favorite, .."!\n"]);
};
report_niceness = favorite => std::print([..favorite, .." is really nice!\n"]);
report_end ==> std::print("done!\n");
```

Notice `==>` is really `=` and `=>`.

## Body

The body is evaluated on every call, if you wish to include more complex expression, you can use block.

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
    std::print([.."Hello ", ..name]);
}
say_hello();
```

## Move

Functions uses `move`, this is thoroughly explained at TODO link to function's move semantic. Notice that `move` is a keyword, it won't be used as parameter, in the example below, the function don't really have a parameter.

```butter
message = "hello";
say_message = move => std::print(message);
```
