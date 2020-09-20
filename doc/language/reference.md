# Reference

Reference borrows and holds an address to a value. You can use `&` to get a reference to a value.

```butter
foo = &100;
```

## Dereferencing

You can dereference via `*`. This gets the underlying value of the reference.

```butter
bar = *foo;
```

Many operations can work with references and automatically dereferences. This includes the following.

- Operands of arithmetic, bitwise, comparison, and logical operations
- Conditional value in if and while control flow
- Field access
- Indexing
- Slicing
- Length access
