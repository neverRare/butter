# Operator precedence

Butter defines the following operator precedence from strongest to weakest.

- `.` `?.` element access or slice `[...]` `?[...]` function call `(...)`
- unary `+` `-` `!` `&` `clone`
- `*` `/` `//` `%`
- `+` `-` `++`
- `==` `!=` `<` `>` `<=` `>=`
- `&` `&&`
- `|` `||`
- `??`
- `<-`
- `=` `return` `break` `=>` `break ... =`

These determines how chain of operations are parsed. You can think of precedence as "binding power": `1 + 3 * 2` is parsed as `1 + (3 * 2)` instead of `(1 + 3) * 2` because `*` binds stronger than `+`, if the latter case is preferred, use parentheses.
