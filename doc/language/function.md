# Function

Function is a reusable expression that are executed on every call. It is defined by 2 parts, the parameters and the body.

```butter
reverse = (arr) => {
    reverse = [];
    for elem in orig {
        reverse <- [elem, ..reverse];
    }
    reverse
};
reverse_in_place = (arr) => {
    orig = *arr;
    new = [];
    for elem in orig {
        new <- [elem, ..new];
    }
    *arr <- new;
};
```

## Parameters

The parameter have the same syntax as struct destructuring.

## Body

The body is evaluated on every call, if you wish to include more complex expression, you can use block.

## Calling

TODO
