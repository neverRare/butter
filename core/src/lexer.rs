pub enum Num {
    UInt(u64),
    Int(i64),
    Float(f64),
}
pub enum Bracket {
    Paren,
    Bracket,
    Brace,
}
pub enum Opening {
    Open,
    Close,
}
pub enum Keyword {
    True,
    False,
    Null,
    If,
    Else,
    Match,
    For,
    Loop,
    While,
}
pub enum Separator {
    Comma,
    Semicolon,
}
pub enum Operator {
    Equal,
    DoubleEqual,
    NotEqual,
    Colon,
    DoubleColon,
    Dot,
    DoubleDot,
    Plus,
    Minus,
    Asterisk,
    Slash,
    DoubleSlash,
    Percent,
    Bang,
    Amp,
    Pipe,
    Caret,
    Tilde,
    DoubleAmp,
    DoublePipe,
    Greater,
    Less,
    DoubleGreater,
    DoubleLess,
    GreaterEqual,
    LessEqual,
    LeftArrow,
    RightArrow,
    RightThickArrow,
    Question,
    DoubleQuestion,
}
pub enum Token<'a> {
    Num(Num),
    Str(Vec<u8>),
    Char(u8),
    Keyword(Keyword),
    Identifier(&'a str),
    Separator(Separator),
    Bracket(Opening, Bracket),
    Operator(Operator),
}
impl<'a> Token<'a> {
    pub fn lex(src: &'a str) -> Vec<Self> {
        todo!()
    }
}
