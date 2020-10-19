# Clone

Clone creates a deep-copy of a value.

```butter
foo = 10;
bar = clone foo;
foo <- 20;
std::assert(bar == 10);
```

Additionally, you can't clone a function.
