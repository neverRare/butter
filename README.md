# ![Butter](butter_text_only.svg)

[Documents](doc/README.md) | [Contributing Guidelines](CONTRIBUTING.md) | [Join the Discord server](https://discord.gg/U75vxW5scB)

A tasty language for building efficient software.

**Note:** Still work in progress and experimental. [Contributions](CONTRIBUTING.md) are welcome.

## A small taste

```butter
-- reverses an array in place
reverse(mut arr) => {
    len = arr^.len;
    for i in [0.< len // 2] {
        mut elem = &arr^[i];
        mut opposite = &arr^[len - i - 1];
        elem^, opposite^ <- >opposite^, >elem^;
    }
}
```

## Goals and planned features

The goals for Butter is to be:

- Concise: The language constructs (aka the syntax) should be simple and have a feel of scripting language.
- High-level: The language should be easily understandable and lacks visible low-level concepts.
- Efficient: Compiled programs should be fast and memory-efficient as much as possible.

For now, Butter is designed to be as simple as possible while being quite useful. Some features such as modules, traits, concurrency/parallelism, comprehensive standard library, and good compiler messages are not a priority for now.

Here are the prioritized features in no particular order:

- Structural typing
- Explicit or inferred mutability
- An extension of Hindley-Milner type system, this includes row-polymorphism
- Refinement types
- Ownership systems with partial automatic reference counting
- Reference types with strict no mutable aliasing rule
- Lifetime analysis
- Low-level representation heuristics &mdash; as an example, the compiler will try to infer if such array can be just a stack array or if it needs to be allocated on heap. Refinement type is used to check if such array exceeds certain capacity

These features ensures the goals and at the same time harms it, there should be a good balance.

After these features are sufficiently implemented. Other useful missing features shall be implemented as well.

## Road map

![A road map for the Butter programming language, depicted in the form style a retro video game map. A single road snakes from right-to-left, with multiple stops, ending with a causeway leading to a castle in the middle of a lake. A flag is placed on the first stop. The project mascot (a cute yellow bear) is at the second stop and exclaims "Let's Go!". The stops on the road are: parser, Hindley-Milner with row polymorphism, IR, lifetime analysis, refinement types, and finally LLVM. An enormous dragon looms behind the castle.](./roadmap.webp)

[More details](https://github.com/neverRare/butter/projects/1)

## Chat with us

You may join and reach us at [the official Discord server](https://discord.gg/U75vxW5scB), it is still quite fresh. You may also join the [/r/ProgrammingLanguages Discord server](https://discord.gg/4Kjt3ZE). It is a helpful and friendly community for people interested in building programming languages, compilers, and stuffs. It have a dedicated #butter channel.
