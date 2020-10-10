# Butter [![Rust](https://github.com/neverRare/butter/workflows/Rust/badge.svg)](https://github.com/neverRare/butter/actions?query=workflow%3ARust)

[Documents](doc/README.md)

Butter aims to be a concise and friendly language for building efficient software.

**Note:** Still work in progress.

## A small taste

```butter
-- reverses an array
reverse = arr => {
    reverse = [];
    for elem in arr {
        reverse <- [elem, ..reverse];
    }
    reverse
};
```

## Goals

- Should be simple, don't complicate.
- Should make sense for user, no low-level shenanigans exposed.
- Should produce efficient software.
