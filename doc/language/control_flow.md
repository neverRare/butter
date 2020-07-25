# Control flow

Almost everything in butter is an expression, even most control flows.

## Block

Block groups statement and creates scope for declaration.

```butter
{
    foo = "hello world";
    len = foo.len;
}
-- foo and len can't be used from here
```

If the last statement is an expression and ommited the semicolon after it, the block can be used as expression and it will evaluate to the last expression. Butter doesn't have uninitialized variable. Although, it could be simulated with null, using blocks can do better job.

```butter
len = {
    foo = "hello world";
    foo.len
};
```

## If

If block is used to conditionally execute statements.

```butter
input = std::get_line();
if input.len == 0 {
    std::print("please input something meaningful\n");
}
```

TODO

## For

TODO

## While

TODO

## Loop

TODO
