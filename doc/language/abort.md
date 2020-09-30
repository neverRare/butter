# Abort

Abort immediately exit the execution of program and indicates an error has occurred.

```butter
abort;
```

## Panicking operators

Few operators can cause errors, these are `/`, `//`, `%`, for division/modulo to 0 and element access or slice `[...]` for out-of-bound error.

These errors are checked at compile-time, but you can choose to make errors happen at runtime with it's panicking counterparts, which are `/!`, `//!`, `%!`, and `[...!]`. When error happens, the program will be immediately aborted.

```butter
fruits = ["strawberry", "pineapple", "banana"];
favorite_fruit = fruits[3!];  -- there's no 4th fruit. oh no, its an error!

-- this doesn't get printed, the program is already aborted upon the previous statement
std::println([.."my favorite fruit is ", ..favorite_fruit]);
```
