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

## Mutable reference

You can mutably borrow a value which lets you mutate the underlying referencing value with `&mut` operator. The variable where the value is borrowed and the variable that would hold the mutable reference (if there's any) must be marked as mutable.

```butter
mut foo = 10;
mut bar = &mut foo;
bar^ <- 20;
std.assert(foo == 10);
```
