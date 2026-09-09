// src/tokens/token_kind.rs
//! Token kind definitions and core token type enumeration.

use crate::tokens::number::Number;
use logos::Logos;
use std::sync::Arc;

use crate::lex::error::LexError;
use crate::tokens::parsers::base::{parse_binary, parse_hex, parse_octal};
use crate::tokens::parsers::numeric::parse_number;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexError)]
pub enum TokenKind {
    #[token("&=")]
    AndEqual,
    #[token("|=")]
    OrEqual,
    #[token("<<=")]
    ShiftLeftEqual,
    #[token(">>=")]
    ShiftRightEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("==")]
    EqualEqual,
    #[token("!=")]
    NotEqual,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("++")]
    PlusPlus,
    #[token("--")]
    MinusMinus,
    #[token("||")]
    OrOr,
    #[token("&&")]
    AndAnd,
    #[token("<<")]
    ShiftLeft,
    #[token(">>")]
    ShiftRight,
    #[token("%=")]
    PercentEqual,
    #[token("^=")]
    XorEqual,
    #[token("~")]
    BitwiseNot,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("!")]
    Not,
    #[token("^")]
    Xor,
    #[token("%")]
    Percent,
    #[token("|")]
    Or,
    #[token("&")]
    And,
    #[token("=")]
    Equal,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    #[token("fun")]
    KeywordFun,
    #[token("if")]
    KeywordIf,
    #[token("else")]
    KeywordElse,
    #[token("return")]
    KeywordReturn,
    #[token("while")]
    KeywordWhile,
    #[token("for")]
    KeywordFor,
    #[token("main")]
    KeywordMain,
    #[token("var")]
    KeywordVar,
    #[token("const")]
    KeywordConst,
    #[token("nullptr")]
    KeywordNullptr,
    #[token("break")]
    KeywordBreak,
    #[token("continue")]
    KeywordContinue,
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    KeywordBool(bool),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Arc::from(lex.slice()), priority = 2)]
    IdentifierAscii(Arc<str>),
    #[regex(r"[\p{Letter}\p{Mark}_][\p{Letter}\p{Mark}\p{Number}_]*", |lex| Arc::from(lex.slice()), priority = 1)]
    IdentifierUnicode(Arc<str>),

    // Consume the complete decimal candidate so malformed suffixes remain
    // attached to the number and are diagnosed by the numeric suffix parser.
    #[regex(r"(?:\d+(?:\.\d+)?|\.\d+)(?:[eE][+-]?\d+)?[A-Za-z0-9]*", parse_number, priority = 10)]
    Numeric(Number),

    #[regex(r"#b[0-9A-Za-z_]+[uU]?", parse_binary, priority = 8)]
    Binary(Number),
    #[regex(r"#o[0-7A-Za-z_]+[uU]?", parse_octal, priority = 8)]
    Octal(Number),
    #[regex(r"#x[0-9A-Fa-fG-Zg-z_]+[uU]?", parse_hex, priority = 8)]
    Hexadecimal(Number),

    #[token("#b", callback = invalid_binary)]
    #[token("#o", callback = invalid_octal)]
    #[token("#x", callback = invalid_hex)]
    InvalidBaseNumber,

    #[regex(r#""([^"\\\r\n]|\\.)*""#, |lex| {
        let slice = lex.slice();
        Arc::from(&slice[1..slice.len() - 1])
    }, priority = 10)]
    StringLiteral(Arc<str>),
    #[regex(r#""([^"\\\r\n]|\\.)*\z"#, callback = unterminated_string, priority = 1)]
    UnterminatedString,
    #[regex(r#"'([^'\\\r\n]|\\.)'"#, |lex| {
        let slice = lex.slice();
        Arc::from(&slice[1..slice.len() - 1])
    }, priority = 10)]
    CharLiteral(Arc<str>),
    #[regex(r#"'([^'\\\r\n]|\\.)*\z"#, callback = unterminated_char, priority = 1)]
    UnterminatedChar,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token(";")]
    Semicolon,

    #[token("i8")]
    TypeI8,
    #[token("i16")]
    TypeI16,
    #[token("i32")]
    TypeI32,
    #[token("i64")]
    TypeI64,
    #[token("u8")]
    TypeU8,
    #[token("u16")]
    TypeU16,
    #[token("u32")]
    TypeU32,
    #[token("u64")]
    TypeU64,
    #[token("f32")]
    TypeF32,
    #[token("f64")]
    TypeF64,
    #[token("char")]
    TypeChar,
    #[token("string")]
    TypeString,
    #[token("bool")]
    TypeBool,

    #[regex(r"\p{White_Space}+", logos::skip)]
    Whitespace,
    #[regex(r"//[^\n\r\u{000B}\u{000C}\u{0085}\u{2028}\u{2029}]*", logos::skip, allow_greedy = true)]
    Comment,
    #[regex(r"/\*[^*]*\*+(?:[^*/][^*]*\*+)*/", logos::skip, priority = 10)]
    MultilineComment,
    #[regex(r"/\*(?:[^*]|\*+[^*/])*\**\z", callback = unterminated_comment, priority = 1)]
    UnterminatedComment,

    Eof,
}

const fn invalid_binary(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> {
    Err(LexError::MalformedBinary)
}
const fn invalid_octal(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> {
    Err(LexError::MalformedOctal)
}
const fn invalid_hex(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> {
    Err(LexError::MalformedHexadecimal)
}
const fn unterminated_string(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> {
    Err(LexError::UnterminatedString)
}
const fn unterminated_char(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> {
    Err(LexError::UnterminatedChar)
}
const fn unterminated_comment(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> {
    Err(LexError::UnterminatedComment)
}

impl TokenKind {
    #[must_use]
    pub const fn is_type(&self) -> bool {
        matches!(
            self,
            Self::TypeI8
                | Self::TypeI16
                | Self::TypeI32
                | Self::TypeI64
                | Self::TypeU8
                | Self::TypeU16
                | Self::TypeU32
                | Self::TypeU64
                | Self::TypeF32
                | Self::TypeF64
                | Self::TypeChar
                | Self::TypeString
                | Self::TypeBool
        )
    }
}
