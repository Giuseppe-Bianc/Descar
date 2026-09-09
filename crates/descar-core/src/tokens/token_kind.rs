// src/tokens/token_kind.rs
//! Token kind definitions and core token type enumeration.
//!
//! This module provides the `TokenKind` enum which represents all possible
//! lexical token types in the language, generated using the Logos lexer library.

use crate::tokens::number::Number;
use logos::Logos;
use std::fmt;
use std::sync::Arc;

use crate::lex::error::LexError;
use crate::tokens::parsers::base::{parse_binary, parse_hex, parse_octal};
use crate::tokens::parsers::numeric::parse_number;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexError)]
pub enum TokenKind {
    #[token("&=")] AndEqual,
    #[token("|=")] OrEqual,
    #[token("<<=")] ShiftLeftEqual,
    #[token(">>=")] ShiftRightEqual,
    #[token("*=")] StarEqual,
    #[token("/=")] SlashEqual,
    #[token("+=")] PlusEqual,
    #[token("-=")] MinusEqual,
    #[token("==")] EqualEqual,
    #[token("!=")] NotEqual,
    #[token("<=")] LessEqual,
    #[token(">=")] GreaterEqual,
    #[token("++")] PlusPlus,
    #[token("--")] MinusMinus,
    #[token("||")] OrOr,
    #[token("&&")] AndAnd,
    #[token("<<")] ShiftLeft,
    #[token(">>")] ShiftRight,
    #[token("%=")] PercentEqual,
    #[token("^=")] XorEqual,
    #[token("~")] BitwiseNot,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token("<")] Less,
    #[token(">") ] Greater,
    #[token("!")] Not,
    #[token("^")] Xor,
    #[token("%")] Percent,
    #[token("|")] Or,
    #[token("&")] And,
    #[token("=")] Equal,
    #[token(":")] Colon,
    #[token(",")] Comma,
    #[token(".")] Dot,

    #[token("fun")] KeywordFun,
    #[token("if")] KeywordIf,
    #[token("else")] KeywordElse,
    #[token("return")] KeywordReturn,
    #[token("while")] KeywordWhile,
    #[token("for")] KeywordFor,
    #[token("main")] KeywordMain,
    #[token("var")] KeywordVar,
    #[token("const")] KeywordConst,
    #[token("nullptr")] KeywordNullptr,
    #[token("break")] KeywordBreak,
    #[token("continue")] KeywordContinue,
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    KeywordBool(bool),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Arc::from(lex.slice()), priority = 2)]
    IdentifierAscii(Arc<str>),
    #[regex(r"[\p{Letter}\p{Mark}_][\p{Letter}\p{Mark}\p{Number}_]*", |lex| Arc::from(lex.slice()), priority = 1)]
    IdentifierUnicode(Arc<str>),

    // The complete decimal candidate, including any trailing alphabetic text,
    // is passed to the suffix parser. This keeps malformed suffixes such as
    // `100i64` and `100u64` together for one diagnostic.
    #[regex(r"(?:\d+(?:\.\d+)?|\.\d+)(?:[eE][+-]?\d+)?[A-Za-z]*", parse_number, priority = 10)]
    Numeric(Number),

    #[regex(r"#b[0-9A-Za-z_]+[uU]?", parse_binary, priority = 8)]
    Binary(Number),
    #[regex(r"#o[0-9A-Za-z_]+[uU]?", parse_octal, priority = 8)]
    Octal(Number),
    #[regex(r"#x[0-9A-Za-z_]+[uU]?", parse_hex, priority = 8)]
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

    #[token("(")] OpenParen,
    #[token(")")] CloseParen,
    #[token("[")] OpenBracket,
    #[token("]")] CloseBracket,
    #[token("{")] OpenBrace,
    #[token("}")] CloseBrace,
    #[token(";")] Semicolon,

    #[token("i8")] TypeI8,
    #[token("i16")] TypeI16,
    #[token("i32")] TypeI32,
    #[token("i64")] TypeI64,
    #[token("u8")] TypeU8,
    #[token("u16")] TypeU16,
    #[token("u32")] TypeU32,
    #[token("u64")] TypeU64,
    #[token("f32")] TypeF32,
    #[token("f64")] TypeF64,
    #[token("char")] TypeChar,
    #[token("string")] TypeString,
    #[token("bool")] TypeBool,

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

const fn invalid_binary(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> { Err(LexError::MalformedBinary) }
const fn invalid_octal(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> { Err(LexError::MalformedOctal) }
const fn invalid_hex(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> { Err(LexError::MalformedHexadecimal) }
const fn unterminated_string(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> { Err(LexError::UnterminatedString) }
const fn unterminated_char(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> { Err(LexError::UnterminatedChar) }
const fn unterminated_comment(_: &mut logos::Lexer<TokenKind>) -> Result<(), LexError> { Err(LexError::UnterminatedComment) }

impl TokenKind {
    #[must_use]
    pub const fn is_type(&self) -> bool {
        matches!(self, Self::TypeI8 | Self::TypeI16 | Self::TypeI32 | Self::TypeI64 | Self::TypeU8 | Self::TypeU16 | Self::TypeU32 | Self::TypeU64 | Self::TypeF32 | Self::TypeF64 | Self::TypeChar | Self::TypeString | Self::TypeBool)
    }
}

impl fmt::Display for TokenKind {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AndEqual => f.write_str("'&='") , Self::OrEqual => f.write_str("'|='") , Self::ShiftLeftEqual => f.write_str("'<<='") , Self::ShiftRightEqual => f.write_str("'>>='") , Self::StarEqual => f.write_str("'*='") , Self::SlashEqual => f.write_str("'/='") , Self::PlusEqual => f.write_str("'+='") , Self::MinusEqual => f.write_str("'-='") , Self::EqualEqual => f.write_str("'=='") , Self::NotEqual => f.write_str("'!='") , Self::LessEqual => f.write_str("'<='") , Self::GreaterEqual => f.write_str("'>='") , Self::PlusPlus => f.write_str("'++'") , Self::MinusMinus => f.write_str("'--'") , Self::OrOr => f.write_str("'||'") , Self::AndAnd => f.write_str("'&&'") , Self::ShiftLeft => f.write_str("'<<'") , Self::ShiftRight => f.write_str("'>>'") , Self::PercentEqual => f.write_str("'%='") , Self::XorEqual => f.write_str("'^='") , Self::BitwiseNot => f.write_str("'~'") , Self::Plus => f.write_str("'+'") , Self::Minus => f.write_str("'-'") , Self::Star => f.write_str("'*'") , Self::Slash => f.write_str("'/'") , Self::Less => f.write_str("'<'") , Self::Greater => f.write_str("'> '") , Self::Not => f.write_str("'!'") , Self::Xor => f.write_str("'^'") , Self::Percent => f.write_str("'%'") , Self::Or => f.write_str("'|'") , Self::And => f.write_str("'&'") , Self::Equal => f.write_str("'='") , Self::Colon => f.write_str("':'") , Self::Comma => f.write_str("','") , Self::Dot => f.write_str("'.'") ,
            Self::KeywordFun => f.write_str("'fun'"), Self::KeywordIf => f.write_str("'if'"), Self::KeywordElse => f.write_str("'else'"), Self::KeywordReturn => f.write_str("'return'"), Self::KeywordWhile => f.write_str("'while'"), Self::KeywordFor => f.write_str("'for'"), Self::KeywordMain => f.write_str("'main'"), Self::KeywordVar => f.write_str("'var'"), Self::KeywordConst => f.write_str("'const'"), Self::KeywordNullptr => f.write_str("'nullptr'"), Self::KeywordBreak => f.write_str("'break'"), Self::KeywordContinue => f.write_str("'continue'"),
            Self::KeywordBool(value) => write!(f, "boolean '{value}'"),
            Self::IdentifierAscii(value) | Self::IdentifierUnicode(value) => write!(f, "identifier '{value}'"),
            Self::Numeric(value) => write!(f, "number '{value}'"),
            Self::Binary(value) => write!(f, "binary '{value}'"), Self::Octal(value) => write!(f, "octal '{value}'"), Self::Hexadecimal(value) => write!(f, "hexadecimal '{value}'"),
            Self::InvalidBaseNumber => f.write_str("invalid base number"),
            Self::StringLiteral(value) => write!(f, "string literal \"{value}\""), Self::UnterminatedString => f.write_str("unterminated string literal"), Self::CharLiteral(value) => write!(f, "character literal '{value}'"), Self::UnterminatedChar => f.write_str("unterminated character literal"),
            Self::OpenParen => f.write_str("'('") , Self::CloseParen => f.write_str("')'") , Self::OpenBracket => f.write_str("'['") , Self::CloseBracket => f.write_str("']'") , Self::OpenBrace => f.write_str("'{'") , Self::CloseBrace => f.write_str("'}'") , Self::Semicolon => f.write_str("';'") ,
            Self::TypeI8 => f.write_str("'i8'"), Self::TypeI16 => f.write_str("'i16'"), Self::TypeI32 => f.write_str("'i32'"), Self::TypeI64 => f.write_str("'i64'"), Self::TypeU8 => f.write_str("'u8'"), Self::TypeU16 => f.write_str("'u16'"), Self::TypeU32 => f.write_str("'u32'"), Self::TypeU64 => f.write_str("'u64'"), Self::TypeF32 => f.write_str("'f32'"), Self::TypeF64 => f.write_str("'f64'"), Self::TypeChar => f.write_str("'char'"), Self::TypeString => f.write_str("'string'"), Self::TypeBool => f.write_str("'bool'"),
            Self::Whitespace => f.write_str("whitespace"), Self::Comment => f.write_str("comment"), Self::MultilineComment => f.write_str("multiline comment"), Self::UnterminatedComment => f.write_str("unterminated multiline comment"), Self::Eof => f.write_str("end of file"),
        }
    }
}
