# Clone

Clone creates a deep-copy of a value.

TODO: this could be an operator rather than a keyword.

```butter
foo = 10;
bar = clone foo;
foo <- 20;
std::assert(bar == 10);
```

Clone performs automatic dereference. However, it doesn't deeply clone references that are withing structs or array, it will only copy its address and will refer to the same value.

```butter
foo = 10;
bar = &foo;
baz = clone baz;  -- this will dereference and copies 10
foo <- 20;
std::assert(baz == 10);
```

Additionally, you can't clone a function.
