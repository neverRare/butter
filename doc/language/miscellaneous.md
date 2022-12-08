# Miscellaneous

## Comment

Comments are used to add few notes or description to Butter code, they are ignored by the compiler.

```butter
-- this is a comment
```

## Keyword reference

These are keywords with special meaning, it cannot be used as variable name, function name, field name, nor tag name.

- `_`
- `break`
- `clone`
- `continue`
- `else`
- `false`
- `for`
- `if`
- `imm`
- `in`
- `loop`
- `match`
- `mut`
- `ref`
- `return`
- `true`
- `while`

## Contextual keyword reference

These are keywords that only have special meaning in some context.

- `len` &ndash; only a keyword when used as field name.

## Punctuation reference

|    Punctuation    | Purpose                              |
| :---------------: | ------------------------------------ |
|        `;`        | Statements terminator                |
|        `,`        | Separator of various kind            |
|        `=`        | Declaration, field                   |
|       `==`        | Equal                                |
|       `!=`        | Not equal                            |
|        `.`        | Field access                         |
|        `^`        | Dereference                          |
|       `..`        | Inclusive range                      |
|       `.<`        | Inclusive-exclusive range            |
|       `>.`        | Exclusive-inclusive range            |
|       `><`        | Exclusive range                      |
|        `+`        | Addition                             |
|       `++`        | Concatenation                        |
|        `-`        | Subtraction                          |
|        `*`        | Multiplication, splat, rest          |
|        `/`        | Unary Minus, division, lifetime      |
|       `//`        | Floor division                       |
|        `%`        | Modulo                               |
|        `!`        | Not                                  |
|  <code>\|</code>  | OR                                   |
| <code>\|\|</code> | Lazy OR                              |
|        `&`        | AND, reference                       |
|       `&&`        | Lazy AND, double reference           |
|        `>`        | Greater than, Move                   |
|        `<`        | Less than                            |
|       `>=`        | Greater than or equal                |
|       `<=`        | Less than or equal                   |
|       `<-`        | Assignment                           |
|       `=>`        | Function, match arm                  |
|        `@`        | Tag                                  |
|        `:`        | Type annotation, mutability modifier |
|       `->`        | Return type annotation               |

## Delimiter reference

| Delimiter | Purpose                                   |
| :-------: | ----------------------------------------- |
|  `(...)`  | Group, record, tuple, parameter, argument |
|  `[...]`  | Array, index, slice                       |
|  `{...}`  | Block, control-flow body                  |

## Operator precedence

Butter defines the following operator precedence from strongest to weakest.

- `.` element access or slice `[...]` function call `(...)` `^`
- unary `-` `!` `&` `@...` `>` `clone`
- `*` `/` `//` `%`
- `+` `-` `++`
- `==` `!=` `<` `>` `<=` `>=`
- `&` `&&`
- `|` `||`
- `<-`
- `return` `break` `(...) =>`
- Type annotation `: ...`

These determines how chain of operations are parsed. You can think of precedence as "binding power": `1 + 3 * 2` is parsed as `1 + (3 * 2)` instead of `(1 + 3) * 2` because `*` binds stronger than `+`, if the latter case is preferred, use [group].

[group]: ./group.md

## Hashbang

[Hashbang] is used to specify interpreter when directly executed in unix-like systems. Butter ignores this like a comment in recognition of its purpose. Hashbangs may only be placed in the beginning of the code without any whitespace before it.

```butter
#!/path/to/interpreter
```

[hashbang]: https://en.wikipedia.org/wiki/Shebang_(Unix)
