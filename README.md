# ![Butter](butter_text_only.svg)

[Documents](doc/README.md) | [Contributing Guidelines](CONTRIBUTING.md) | [Discord server](https://discord.gg/U75vxW5scB)

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
- Explicit: There must be little-to-no vagueness syntax-wise or semantic-wise.
- High-level: The language should be easily understandable and lacks visible low-level concepts.
- Efficient: Compiled programs should be fast and memory-efficient as much as possible.

And here are the features to be implemented:

- Structural typing
- Mix of explicit and inferred mutability &mdash; variables are explicitly "mut-typed" but it's inner data such as individual record fields are "mut-type inferred"
- An extension of Hindley-Milner type system, this includes row-polymorphism
- Refinement types
- Mix of ownership systems and automatic reference counting &mdash; data that are immutable and never moved are shareable
- Reference types with mutability xor aliasability rule
- Lifetime analysis

More features planned:

- Traits or typeclasses
- `newtype` for nominally typed data
- Module and visibility system
- Shareable mutable containers &mdash; those are also interiorly mutable
- Low-level representation heuristics &mdash; as an example, the compiler will try to infer if such array can be just a stack array or if it needs to be allocated on heap. Refinement type is used to check if such array exceeds certain capacity

These features ensures some goals and at the same harms some, there should be a good balance.Ultimately, the designs and features are up for my decision and taste and there are influences from Rust, Typescript, and Haskell.

## Road map

![A road map for the Butter programming language, depicted in the form style a retro video game map. A single road snakes from right-to-left, with multiple stops, ending with a causeway leading to a castle in the middle of a lake. A flag is placed on the first stop. The project mascot (a cute yellow bear) is at the second stop and exclaims "Let's Go!". The stops on the road are: parser, Hindley-Milner with row polymorphism, IR, lifetime analysis, refinement types, and finally LLVM. An enormous dragon looms behind the castle.](./roadmap.webp)

[More details](https://github.com/neverRare/butter/projects/1)

## Chat with us

You may join and reach us at [the official Discord server](https://discord.gg/U75vxW5scB). You may also join the [/r/ProgrammingLanguages Discord server](https://discord.gg/4Kjt3ZE), it have a dedicated #butter channel.
