# Miscellaneous

## Keyword reference

| Keywords   | Purpose  |
| ---------- | -------- |
| `break`    | Break    |
| `clone`    | Clone    |
| `continue` | Continue |
| `else`     | If       |
| `false`    | False    |
| `for`      | For      |
| `if`       | If       |
| `in`       | For      |
| `loop`     | Loop     |
| `null`     | Null     |
| `return`   | Return   |
| `true`     | True     |
| `while`    | While    |

## Punctuation reference

| Punctuation | Purpose                     |
| ----------- | --------------------------- |
| `=`         | Declaration, Field, Break   |
| `==`        | Equal                       |
| `!=`        | Not equal                   |
| `.`         | Field access                |
| `..`        | Inclusive range             |
| `.<`        | Inclusive-exclusive range   |
| `>.`        | Exclusive-inclusive range   |
| `><`        | Exclusive range             |
| `+`         | Unary Plus, Addition        |
| `++`        | Concatenation               |
| `-`         | Subtraction                 |
| `*`         | Multiplication, Splat, Rest |
| `/`         | Unary Minus, Division       |
| `//`        | Floor Division              |
| `%`         | Modulo                      |
| `!`         | Not                         |
| `|`         | OR                          |
| `||`        | Lazy OR                     |
| `&`         | AND, reference              |
| `&&`        | Lazy AND, Double reference  |
| `>`         | Greater than                |
| `<`         | Less than                   |
| `>=`        | Greater than or equal       |
| `<=`        | Less than or equal          |
| `<-`        | Assignment                  |
| `=>`        | Function                    |
| `?`         | Optional chain              |
| `??`        | Null coalescing             |

## Delimiter reference

| Delimiter | Purpose                             |
| --------- | ----------------------------------- |
| `(...)`   | Group, Struct,  Parameter, Argument |
| `[...]`   | Array, Index, Slice                 |
| `{...}`   | Block, Control-flow body            |

## Separator references

| Separator | Operand                                                                     |
| --------- | --------------------------------------------------------------------------- |
| `,`       | Elements, Fields, Parameters, Arguments, Expressions of parallel assignment |
| `;`       | Statements                                                                  |

## Operator precedence

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
- `=` `return` `break ... =` `=>`

These determines how chain of operations are parsed. You can think of precedence as "binding power": `1 + 3 * 2` is parsed as `1 + (3 * 2)` instead of `(1 + 3) * 2` because `*` binds stronger than `+`, if the latter case is preferred, use parentheses.
