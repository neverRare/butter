pub mod lexer;

#[macro_export]
macro_rules! match_lex {
    ($src:expr; $(Some($pat:pat) => $expr:expr,)* else $last_pat:pat => $last_expr:expr $(,)?) => {{
        let src = $src;
        $(
            if let Some((step, $pat)) = $crate::lexer::Lex::lex_first(src) {
                Some((step, $expr))
            } else
        )* {
            let $last_pat = src;
            $last_expr
        }
    }};
}
