# Block

Block groups statement and creates scope for declaration.

```butter
{
    foo = "hello world";
    len = foo.len;
}
-- foo and len can't be used from here
```

If the last statement is an expression and have no semicolon after it, the block can be used as expression and it will evaluate to the last expression.

```butter
len = {
    foo = "hello world";
    foo.len
};
```

## Breakable block

TODO: explanation

```butter
num = with break {
    if foo == 10 {
        break 10;
    }
    20
};
```
