# Miscellaneous

## Keyword reference

These are keywords with special meaning, it cannot be used as variable name, function name, field name, nor tag name.

- `break`
- `clone`
- `continue`
- `else`
- `false`
- `for`
- `if`
- `in`
- `loop`
- `match`
- `mut`
- `ref`
- `return`
- `true`
- `void`
- `while`

## Contextual keyword reference

These are keywords that only have special meaning in some context.

- `len` &ndash; only a keyword when used as field name.

## Punctuation reference

| Punctuation | Purpose                     |
| ----------- | --------------------------- |
| `=`         | Declaration, Field, Break   |
| `==`        | Equal                       |
| `!=`        | Not equal                   |
| `.`         | Field access                |
| `^`         | Dereference                 |
| `..`        | Inclusive range             |
| `.<`        | Inclusive-exclusive range   |
| `>.`        | Exclusive-inclusive range   |
| `><`        | Exclusive range             |
| `+`         | Addition                    |
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
| `>`         | Greater than, Move          |
| `<`         | Less than                   |
| `>=`        | Greater than or equal       |
| `<=`        | Less than or equal          |
| `<-`        | Assignment                  |
| `=>`        | Function                    |
| `@`         | Tag                         |

## Delimiter reference

| Delimiter | Purpose                             |
| --------- | ----------------------------------- |
| `(...)`   | Group, Record,  Parameter, Argument |
| `[...]`   | Array, Index, Slice                 |
| `{...}`   | Block, Control-flow body            |

## Separator references

| Separator | Operand                                                                     |
| --------- | --------------------------------------------------------------------------- |
| `,`       | Elements, Fields, Parameters, Arguments, Expressions of parallel assignment |
| `;`       | Statements                                                                  |

## Operator precedence

Butter defines the following operator precedence from strongest to weakest.

- `.` element access or slice `[...]` function call `(...)` `^`
- unary `+` `-` `!` `&` `@...`
- `*` `/` `//` `%`
- `+` `-` `++`
- `==` `!=` `<` `>` `<=` `>=`
- `&` `&&`
- `|` `||`
- `<-`
- `=` `return` `break ... =` `=>`

These determines how chain of operations are parsed. You can think of precedence as "binding power": `1 + 3 * 2` is parsed as `1 + (3 * 2)` instead of `(1 + 3) * 2` because `*` binds stronger than `+`, if the latter case is preferred, use [group].

[group]: ./group.md

## Hashbang

[Hashbang] is used to specify interpreter when directly executed in unix-like systems. Butter ignores this like a comment in recognition of its purpose. Hashbangs may only be placed in the beginning of the code without any whitespace before it.

```butter
#!/path/to/interpreter
```

[Hashbang]: https://en.wikipedia.org/wiki/Shebang_(Unix)
