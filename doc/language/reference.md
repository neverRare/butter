# Reference

Referencing is performed to share data rather than moving nor cloning it. This is explained thoroughly at TODO link to borrow semantic.

```butter
foo = 10;
bar = &foo;  -- just borrow the data from foo
```

## Indirection and dereference

A reference is an indirection, it holds an address to a value. A reference can even hold multiple indirection. You'll need to use multiple `&` as Butter always perform dereference.

```butter
foo = 10;  -- original value
bar = &foo;  -- bar is a reference to foo
baz = &&bar;  -- baz is a reference to a reference to foo
```

Access of the referencing value, also known as dereferencing, is always performed.

```butter
foo = 10;
bar = &foo;
bar <- 20;  -- since bar is a reference to foo, foo is assigned to 20
```

You can preserve the indirection by using `&`.

```butter
foo = 10;
bar = 20;
baz = &foo; -- baz refers to foo
&baz <- &bar; -- baz now refers to bar
```
