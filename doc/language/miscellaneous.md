# Miscellaneous

## Line comment

TODO: explanation

```butter
-- this is a comment
```

## Multiline comment

Primarily intended for textual comments and documentation comments. The contents must be a well formed markdown document.

```butter
/-
multiline
comment
-/
```

## Ignore codeblock

Primarily intended for disabling span of codes. The contents must be lexable. This means it can be nested.

```butter
{-
print_line("this code is ignored, nothing would be printed");
-}
```

## Raw identifier

This allows you to use keywords and spaces in your variable. It is processed like a regular string literal.

```butter
i"loop" = 10;
```

You can use raw string on raw identifier as well.

```butter
ri"windows\path" = 20;
```

## Keyword reference

These are keywords with special meaning, it cannot be used as variable name, function name, field name, nor tag name.

- `_`
- `and`
- `as`
- `break`
- `continue`
- `else`
- `for`
- `if`
- `imm`
- `in`
- `loop`
- `match`
- `mut`
- `not`
- `once`
- `or`
- `return`
- `share`
- `undef`
- `while`
- `with`

## Contextual keyword reference

These are keywords that only have special meaning in some context.

- `len` &ndash; only a keyword when used as field name.

## Reserved keywords

- `alias`
- `auto`
- `global`
- `impl`
- `lib`
- `mod`
- `newtype`
- `pub`
- `self`
- `super`
- `trait`
- `where`

## Punctuation reference

| Punctuation | Purpose                              |
| :---------: | ------------------------------------ |
|     `;`     | Statements terminator                |
|     `,`     | Separator of various kind            |
|     `=`     | Declaration, field                   |
|    `==`     | Equal                                |
|    `/=`     | Not equal                            |
|     `.`     | Field access                         |
|     `^`     | Dereference                          |
|    `..`     | Inclusive range                      |
|    `.<`     | Inclusive-exclusive range            |
|    `<.`     | Exclusive-inclusive range            |
|    `<<`     | Exclusive range                      |
|     `+`     | Addition, share                      |
|    `++`     | Concatenation                        |
|     `-`     | Subtraction                          |
|     `*`     | Multiplication, splat, rest          |
|     `/`     | Unary Minus, division, lifetime      |
|    `//`     | Floor division                       |
|     `%`     | Modulo                               |
|     `&`     | Reference                            |
|    `&<`     | Bind to reference                    |
|     `>`     | Greater than, Move                   |
|     `<`     | Less than                            |
|    `>=`     | Greater than or equal                |
|    `<=`     | Less than or equal                   |
|    `<-`     | Assignment                           |
|    `=>`     | Function, match arm                  |
|     `@`     | Tag                                  |
|     `:`     | Type annotation, mutability modifier |
|    `->`     | Return type annotation               |

## Delimiter reference

| Delimiter | Purpose                                   |
| :-------: | ----------------------------------------- |
|  `(...)`  | Group, record, tuple, parameter, argument |
|  `[...]`  | Array, index, slice                       |
|  `{...}`  | Block, control-flow body                  |

## Operator precedence

Butter defines the following operator precedence from strongest to weakest.

- `.` element access or slice `[...]` function call `(...)` `^`
- unary `-` `+` `&` `@...` `>` `not`
- `*` `/` `//` `%`
- `+` `-` `++`
- `==` `/=` `<` `>` `<=` `>=`
- `and`
- `or`
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
