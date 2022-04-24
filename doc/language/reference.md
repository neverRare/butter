# Reference

Referencing is performed to borrow data with `&` operator.

```butter
foo = 10;
bar = &foo;  -- just borrow the data from foo
```

## Dereference

Reference is an indirection, it holds an address to a value. You can access the borrowed value by dereferencing via postfix `^` operator.

```butter
foo = 10;
bar = &foo;  -- borrow foo
baz = bar^;  -- access where bar refers to, which is foo
```

TODO: explain lifetime, mutability, and "no shared mutable" rule
