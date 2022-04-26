# Variable Declaration

Variables are declared via `=`.

```butter
foo = 10;
bar = false;
```

Variables can shadow previously declared variable with the same name, either on the same or on upper scope.

```butter
foo = 10;
{
    foo = false;
    std.assert(foo == false);
}
std.assert(foo == 10);
foo = 20;
std.assert(foo == 20);
foo = foo == 20;
std.assert(foo == true);
```
