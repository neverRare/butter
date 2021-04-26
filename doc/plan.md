# Planned stuffs

In order to achieve the design principles of Butter, I have the following plan. Note that these are all just ingredients for now, could always change, and nothing is baked yet.

## Refinement types

Butter will have some sort of refinement types where types also holds possible values. It will only apply to numbers, booleans, and tags of tagged union. Number types will only hold finite union of ranges (If I'm lazy enough, maybe only a range).

With this, we may be able to perform better exhaustive check and catch out-of-bound errors on compile time. Refinement types could be useful for the other features I planned.

## Relaxed ownership system

Inspired from Rust, Butter is planned to have ownership and borrow system. It have the following benefits:

- Ownership system makes program memory safe without much cost
- Borrow system makes it easy to reason out where mutation occurs (compared to other languages with implicit pointer to instances)

However, it can make the language more restrictive and less friendly. So I planned to make it more relaxed:

- There be lifetime inference.
- Immutable data that are never moved to mutable context will have the least restriction. These will be automatically reference counted. Lifetime inference will be used to elide reference count increment/decrement or not use reference counting at all if possible.
- Mutable data together with immutable data that is moved to mutable context will have full blown ownership system and there will be no mutable aliasing. We may use refined types to tell whether multiple slice or index to a single mutable array causes aliasing or not.

## Low-level polymorphism (I just made up this term)

Butter's high-level type will corresponds to multiple low-level types: For example, numbers will be bytes, int, float, etc.; Arrays will be stack array, vector, etc. Butter will use mutability and refined types to get the best, but not the most best, low-level types. For example, integers that are small enough could be stored in bytes; Arrays with length that remain small enough could just be a stack array.
