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

- Concise: The language constructs (aka the syntax) should be simple and free from unnecessary boilerplate.
- Friendly: The language should be easily understandable and lacks visible low-level concepts. (Friendliness of error messages is a non-goal for now)
- Efficient: Compiled programs should be fast and memory-efficient as much as possible.
