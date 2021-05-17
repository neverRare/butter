# ![Butter](butter_text_only.svg)

[Documents](doc/README.md)

Butter aims to be a concise and friendly language for building efficient software.

**Note:** Still work in progress.

## A small taste

```butter
-- reverses an array in place
reverse(mut arr) => {
    for i in [0.< arr^.len // 2] {
        opposite = arr^.len - i - 1;
        arr^[i], arr^[opposite] <- arr^[opposite], arr^[i];
    }
}
```

## Design principle

Butter is designed to be

- Concise: The language constructs (aka the syntax) should be simple and free from unnecessary boilerplate.
- Friendly: The language should be easily understandable and lacks visible low-level concepts. (Friendliness of error messages is a non-goal for now)
- Efficient: Compiled programs should be fast and memory-efficient as much as possible.

Butter is still in development, I have a [plan](./doc/plan.md) to make this possible, hopefully.

## Road map

[![A road map for the Butter programming language, depicted in the form style a retro video game map. A single road snakes from right-to-left, with multiple stops, ending with a causeway leading to a castle in the middle of a lake. A flag is placed on the first stop. The project mascot (a cute yellow bear) is arriving at the second stop. The stops on the road are: parser, Hindley-Milner with row polymorphism and IR, lifetime analysis, refinement types, and finally LLVM. An enormous dragon looms behind the castle.](./roadmap.png "Road map")](https://github.com/neverRare/butter/projects/1)
