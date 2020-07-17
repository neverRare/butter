# Clone

Clone takes a reference of a value and returns a deep-copy of its underlying value. It doesn't deep-copy references, it only copies its address and retains underlying value.

```butter
foo = 10;
bar = clone &foo;
foo <- 20;
std::assert(bar == 10);
```
