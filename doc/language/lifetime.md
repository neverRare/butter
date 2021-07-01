# Lifetime

Lifetime are region of the code at which a value is present on a certain place. In other words, when a place is [initialized].

[initialized]: move_and_initialization.md#initialization

## Move and Assignment

Lifetimes may refer to borken regions of codes. This happens when a value is [moved] from a place. TODO: explain reinitialization.

[moved]: move_and_initialization.md#

TODO example

## End of Scope

Places never lives forever. This happens at the end of the scope where a variable is available.

TODO example
