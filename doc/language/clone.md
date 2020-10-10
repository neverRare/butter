# Clone

Clone creates a deep-copy of a value.

TODO: this could be an operator rather than a keyword.

```butter
foo = 10;
bar = clone foo;
foo <- 20;
std::assert(bar == 10);
```

Clone also deep-copies references. The resulting value will not be a reference and instead, a deep-copy of its underlying value.

```butter
foo = 10;
bar = &foo;
baz = clone baz;  -- this will dereference and copies 10
foo <- 20;
std::assert(baz == 10);
```

Additionally, you can't clone a function.
