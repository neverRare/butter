# Clone

Clone creates a deep-copy of a value.

```butter
foo = 10;
bar = clone foo;
foo <- 20;
std::assert(bar == 10);
```

Clone recursively copies a value. It doesn't deeply copies a reference, it only copies an address and will refer to the same value. It can't clone function and will result in error if you do so.
