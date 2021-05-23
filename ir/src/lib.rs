use id_arena::Arena;
use id_arena::Id;

struct Ir {
    funs: Vec<Fun>,
}
struct Fun {
    type_: (),
    arena: Arena<Op>,
    start: Id<Op>,
}
enum Op {
    Int(u64),
}
