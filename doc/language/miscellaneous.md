# Miscellaneous

## Keyword reference

| Keywords   | Purpose             |
| ---------- | ------------------- |
| `true`     | Boolean literal     |
| `false`    | Boolean literal     |
| `null`     | Null literal        |
| `clone`    | Clone operator      |
| `if`       | If expression       |
| `else`     | If expression       |
| `for`      | For statement       |
| `in`       | For statement       |
| `loop`     | Loop expression     |
| `while`    | While statement     |
| `break`    | Break expression    |
| `continue` | Continue expression |
| `return`   | Return expression   |

## Punctuation reference

| Punctuation | Purpose                                                    |
| ----------- | ---------------------------------------------------------- |
| `=`         | Variable declaration, Break expression                     |
| `==`        | Equal operator                                             |
| `!=`        | Not equal operator                                         |
| `:`         | Struct field                                               |
| `::`        | Path                                                       |
| `.`         | Field access                                               |
| `..`        | Inclusive range                                            |
| `.<`        | Inclusive-exclusive range                                  |
| `>.`        | Exclusive-inclusive range                                  |
| `><`        | Exclusive range                                            |
| `+`         | Addition                                                   |
| `++`        | Concatenation                                              |
| `-`         | Subtraction                                                |
| `*`         | Multiplication, Splat, Rest                                |
| `/`         | Division                                                   |
| `//`        | Floor Division                                             |
| `%`         | Modulo                                                     |
| `!`         | Not operator                                               |
| `&`         | Reference, And operator                                    |
| `|`         | Or operator                                                |
| `&&`        | Lazy and operator                                          |
| `||`        | Lazy or operator                                           |
| `>`         | Greater than operator                                      |
| `<`         | Less than operator                                         |
| `>=`        | Greater than or equal operator                             |
| `<=`        | Less than or equal operator                                |
| `<-`        | Assignment                                                 |
| `=>`        | Function statement and expression                          |
| `?`         | Optional index and slice (`?` and `[` are separate tokens) |
| `?.`        | Optional field access                                      |
| `??`        | Null coalescing                                            |

## Delimiter reference

| Delimiter | Purpose                                                      |
| --------- | ------------------------------------------------------------ |
| `(...)`   | Grouping, Struct, Function parameter, Function call argument |
| `[...]`   | Array expression, Array index, Array slice                   |
| `{...}`   | Block, Control-flow body                                     |

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
- `=` `return` `break` `=>` `break ... =`

These determines how chain of operations are parsed. You can think of precedence as "binding power": `1 + 3 * 2` is parsed as `1 + (3 * 2)` instead of `(1 + 3) * 2` because `*` binds stronger than `+`, if the latter case is preferred, use parentheses.
