# ![Butter](butter_text_only.svg)

[Documents](doc/README.md) | [Contributing Guidelines](CONTRIBUTING.md) | [Discord server](https://discord.gg/U75vxW5scB)

A tasty language for building efficient software.

**Note:** Still work in progress and experimental. [Contributions] are welcome.

[Contributions]: CONTRIBUTING.md

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

## Goals

Butter is a personal and experimental language that seeks balance for these aspects:

- Concise: The language constructs should be simple and have a feel of scripting language.
- Explicit: There should be little-to-no vagueness syntax-wise nor semantic-wise.
- High-level: Low-level concepts that are hard to understand should be abstracted.
- Efficient: The added runtime code for compiled programs should be minimal both in size and runtime impact.
- Correct: Detectable errors should be caught on compile-time.

I also to want to experiment with novel features deemed necessary for these goals such refinement types.

Being my personal project, designs and features are ultimately up for my decision and taste. Some features can help with some aspect while also hurt other, this is where I weigh in the pros and cons. Of course, that doesn't mean I won't listen to suggestions, I'd be happy to hear your thoughts about Butter's design by opening an issue or [chatting with us] through Discord.

[chatting with us]: #chat-with-us

## Road map

![A road map for the Butter programming language, depicted in the form style a retro video game map. A single road snakes from right-to-left, with multiple stops, ending with a causeway leading to a castle in the middle of a lake. A flag is placed on the first stop. The project mascot (a cute yellow bear) is at the second stop and exclaims "Let's Go!". The stops on the road are: parser, Hindley-Milner with row polymorphism, IR, lifetime analysis, refinement types, and finally LLVM. An enormous dragon looms behind the castle.](./roadmap.webp)

[More details](https://github.com/neverRare/butter/projects/1)

## Planned features

Features to be implemented

- Structural typing
- Mix of explicit and inferred mutability &mdash; variables are explicitly "mut-typed" but it's inner data such as individual record fields are "mut-type inferred"
- An extension of Hindley-Milner type system, this includes row-polymorphism
- Refinement types
- Mix of ownership systems and automatic reference counting &mdash; data that are immutable and never moved are shareable, otherwise they are owned
- Reference types with mutability xor aliasability rule
- Lifetime analysis

Features to be implemented later on

- Traits or typeclasses
- `newtype` for nominally typed data
- Module and visibility system
- Shareable mutable containers &mdash; those are also interiorly mutable
- Low-level representation heuristics &mdash; as an example, the compiler will try to infer if such array can be just a stack array or if it needs to be allocated on heap. Refinement type is used to check if such array exceeds certain capacity

## Chat with us

You may join and reach us at [the official Discord server](https://discord.gg/U75vxW5scB). You may also join the [/r/ProgrammingLanguages Discord server](https://discord.gg/4Kjt3ZE), it have a dedicated #butter channel.
