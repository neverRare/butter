# Reference

Referencing is performed to share data rather than moving nor cloning it. This is explained thoroughly at TODO link to borrow semantic.

```butter
foo = 10;
bar = &foo;  -- just borrow the data from foo
```

## Indirection

A reference is an indirection, it holds an address to a value. A reference can even hold multiple indirection.

```butter
foo = 10;  -- original value
bar = &foo;  -- borrow foo, now bar is a reference to foo
baz = &bar;  -- borrow bar, now baz is a reference to a reference to foo
```

Access of the referencing value, also known as dereferencing, is always performed. This is applied even to assignment.

```butter
foo = 10;
bar = &foo;
bar = 20;  -- since bar is a reference to foo, foo is assigned to 20
```

If you wish to change or reassign the reference instead of the value its referring to, use `&`.

```butter
foo = 10;
bar = 20;
baz = &foo; -- baz refers to foo
&baz <- &bar; -- baz now refers to bar
```
