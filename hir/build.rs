use std::env;
use std::path::Path;

fn main() {
    string_cache_codegen::AtomType::new("hir_string_cache::Atom", "keyword!")
        .atoms([
            "_", "break", "clone", "continue", "else", "false", "for", "if", "in", "len", "loop",
            "match", "mut", "ref", "return", "true", "while",
        ])
        .write_to_file(&Path::new(&env::var("OUT_DIR").unwrap()).join("hir_string_cache.rs"))
        .unwrap()
}
