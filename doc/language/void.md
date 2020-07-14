# Void

Void represents nothingness.

## Uninitialized variable

You can use void to simulate unitialized variable, since Butter don't have syntax for it. Void variable can still be written and change its type in flow-sensitive manner.

```butter
foo = void;
-- do things
foo <- "result from things";
-- foo is no longer void from here
```

Keep in mind that there maybe better solution for this.

```butter
foo = {
    -- do things
    "result from things"
};
```
