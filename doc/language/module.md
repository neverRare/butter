# Module

For now, Butter have limited module system. You can only work with one butter file and the only useable library is the standard library `std`.

## Path

Path is an expression to refer to expressions within module. To refer to `print` function within `std`, simply `std::print`.

```butter
std::print("hello world!\n");
```
