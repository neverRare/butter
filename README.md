# ![Butter](butter_text_only.svg)

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

## Design Principle

Butter is designed to be

- Friendly, for experienced and especially for beginners
- Compiled to fast and memory-efficient binary

Butter should have the following

- Simplicity and consistency
- Lack of visible low-level concepts
- Speed and memory-efficiency of resulting software
