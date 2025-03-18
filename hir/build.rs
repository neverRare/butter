use std::env;
use std::path::Path;

fn main() {
    string_cache_codegen::AtomType::new("hir_string_cache::Atom", "keyword!")
        .atoms([
            "_", "alias", "and", "as", "break", "cell", "continue", "else", "for", "if", "imm",
            "impl", "in", "len", "loop", "match", "mod", "mut", "never", "newtype", "not", "once",
            "or", "pub", "return", "share", "trait", "undef", "where", "while",
        ])
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("hir_string_cache.rs"))
        .unwrap()
}
