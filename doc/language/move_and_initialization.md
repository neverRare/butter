# Move

This moves the value to a different place and deinitializes the original place.

```butter
foo = 10;
bar = >foo;  -- move the value 10 to bar
-- now, foo can't be used here
```

## Place

Places are value or part of the value that can be assigned into. These are variables, properties, indices, and dereferences.

TODO: there can be anonymous places, should it be mentioned?

NOTE: I think this better fits to appendix or glossary

## Initialization, deinitialization, and reinitialization

A place is initialized when a value is present in it. It is deinitialized when the value is moved to different place as shown previously. It can be reinitialized with assignment

```butter
mut foo = 10;
bar = >foo;  -- move the value 10 to bar
-- foo can't be used here
foo <- 20;  -- foo is reinitialized with new value, now it can be used again
```

## Move and reference

You are able to take the referencing value of the reference with move operator. This deinitializes the referencing place and it must be reinitialized again before the lifetime of the reference ended.

```butter
mut foo = 10;
mut bar = &foo;
baz <- >bar^; -- moves the value `10` to baz
-- bar can't be used here
bar^ <- 20;  -- reinitialize the underlying place of bar
-- bar's lifetime just ends here
```
