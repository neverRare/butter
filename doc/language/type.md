# Type

Butter have the following types:

- Void
- Number
- Boolean
- Array (homogenous)
- Function
- Reference
- Record
- Tagged Union

Butter is able to infer these and check if type mismatch happens.

```butter
mut foo = 10;  -- foo is inferred as number
foo <- false;  -- type error here, boolean value can't be assigned on it
```
