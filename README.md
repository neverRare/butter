# Butter [![Rust][CI status]][CI Link]

[CI Status]: https://github.com/neverRare/butter/workflows/Rust/badge.svg
[CI Link]: https://github.com/neverRare/butter/actions?query=workflow%3ARust

[Documents](doc/README.md)

Butter aims to be a concise and friendly language for building efficient software.

**Note:** Still work in progress.

## A small taste

```butter
-- reverses an array in place
reverse(arr) => {
    for i in [0.< arr.len // 2] {
        opposite = arr.len - i - 1;
        arr[i], arr[opposite] <- arr[opposite], arr[i];
    }
}
```

## Goals

- Should be simple, don't complicate.
- Should make sense for user, no low-level shenanigans exposed.
- Should produce efficient software.
