# Reference

Referencing is performed to share data rather than moving nor cloning it. This is explained thoroughly at TODO link to borrow semantic.

```butter
foo = 10;
bar = &foo;  -- just borrow the data from foo
```

## Indirection and dereference

A reference is an indirection, it holds an address to a value. Access of the referencing value, also known as dereferencing, is always performed.

```butter
foo = 10;
bar = &foo;
bar <- 20;  -- since bar is a reference to foo, foo is assigned to 20
```

A reference can even hold multiple indirection. You'll need to use multiple `&` as Butter always perform dereference.

```butter
foo = 10;  -- original value
bar = &foo;  -- bar is a reference to foo
baz = &&bar;  -- baz is a reference to a reference to foo
```

You can change the address of the reference with `&` and `<-`.

```butter
foo = 10;
bar = 20;
baz = &foo; -- baz refers to foo
&baz <- &bar; -- baz now refers to bar
```

## Clone

Clone deep-copies references. The resulting value will not be a reference and instead, a deep-copy of its underlying value.

```butter
foo = 10;
bar = &foo;
baz = clone baz;  -- this will dereference and copies 10
foo <- 20;
std::assert(baz == 10);
```
