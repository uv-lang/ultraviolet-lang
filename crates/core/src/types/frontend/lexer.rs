use std::fmt::Display;

use crate::types::frontend::Span;

/// Tokens produced by the UV template lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum UVLexerTokens {
    /// Opening angle bracket `<`.
    OpeningAngleBracket,
    /// Closing angle bracket `>`.
    ClosingAngleBracket,
    /// Self-closing tag suffix `/>`.
    SelfClosingAngleBracket,
    /// Opening bracket of a closing tag `</`.
    OpeningAngleBracketSlash,
    /// Literal — a named token such as a tag name or attribute name.
    Literal(String),
    /// Raw string — arbitrary text content between tags.
    RawString(String),
    /// Unknown character that the lexer could not recognize.
    Unknown(char),
}

impl Display for UVLexerTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UVLexerTokens::OpeningAngleBracket => write!(f, "<"),
            UVLexerTokens::ClosingAngleBracket => write!(f, ">"),
            UVLexerTokens::SelfClosingAngleBracket => write!(f, "/>"),
            UVLexerTokens::OpeningAngleBracketSlash => write!(f, "</"),
            UVLexerTokens::Literal(str) => write!(f, "[Literal \"{str}\"]"),
            UVLexerTokens::RawString(str) => write!(f, "[Raw string \"{str}\"]"),
            UVLexerTokens::Unknown(ch) => write!(f, "{ch}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UVToken {
    pub token: UVLexerTokens,
    pub span: Span,
}

#[derive(PartialEq)]
pub enum LexerParseState {
    Default,
    ParsingRawStringLiteral(Option<String>),
}
