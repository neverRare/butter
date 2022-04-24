# String

Butter have string and char literals, but it is just a syntactic sugar to an [array] of bytes and a byte respectively. Strings are enclosed with `""` while chars are with `''`.

[array]: array.md

```butter
name = "bart";
letter = 'a';
```

## Escaping

| Notation | Output                                                 |
| -------- | ------------------------------------------------------ |
| `\\`     | Backslash                                              |
| `\"`     | Double quote                                           |
| `\'`     | Single quote                                           |
| `\n`     | Line feed                                              |
| `\r`     | Carriage return                                        |
| `\t`     | Horizontal tab                                         |
| `\v`     | Vertical tab                                           |
| `\0`     | Null                                                   |
| `\xXX`   | UTF-8 code unit, where `XX` is the code in hexadecimal |
