use ultraviolet_core::{
    errors::SpannedError,
    types::frontend::{
        Span, Spanned,
        lexer::{UVLexerTokens, UVToken},
        tokens::{UVParseBody, UVParseNode, UVParseState},
    },
};

use crate::iterator::Iter;

/**
Parses a tokens flow to a parse tree
*/
pub struct TokenParser {
    iter: Iter<UVToken>,
}

impl TokenParser {
    /// Create new TokenParser and pass tokens
    pub fn new(tokens: Vec<UVToken>) -> Self {
        Self {
            iter: Iter::from(tokens),
        }
    }

    /// Parse and get Parse Tree
    pub fn parse(&mut self) -> Result<UVParseNode, SpannedError> {
        let mut parse_state = UVParseState::Unknown;
        let mut tag = UVParseNode {
            name: String::new(),
            children: Vec::new(),
            self_closing: false,
            extra_param: String::new(),
            span: Span::default(),
        };

        let mut closing_tag_name = String::new();
        while let Some(token) = self.iter.next() {
            match &token.token {
                UVLexerTokens::OpeningAngleBracket => match parse_state {
                    UVParseState::Unknown => {
                        parse_state = UVParseState::TagName;
                        tag.span.start = token.span.start;
                    },
                    UVParseState::TagBody => {
                        self.iter.step_back();
                        tag.children.push(UVParseBody::Tag(Box::new(self.parse()?)));
                    },
                    _ => {
                        return Err(SpannedError::new("Unexpected `<` token", token.span));
                    },
                },
                UVLexerTokens::ClosingAngleBracket => match parse_state {
                    UVParseState::ClosingAngleBracketOpeningTag | UVParseState::ExtraParam => {
                        parse_state = UVParseState::TagBody
                    },
                    UVParseState::ClosingAngleBracketClosingTag => {
                        if tag.name.ne(&closing_tag_name) {
                            return Err(SpannedError::new(
                                format!("Unexpected closing tag `{}`. Expected `{}`", closing_tag_name, tag.name),
                                Span::new(token.span.start - closing_tag_name.len(), token.span.end - 1),
                            ));
                        }

                        tag.span.end = token.span.end;
                        return Ok(tag);
                    },
                    _ => {
                        return Err(SpannedError::new("Unexpected `>` token", token.span));
                    },
                },
                UVLexerTokens::SelfClosingAngleBracket => match parse_state {
                    UVParseState::ClosingAngleBracketOpeningTag | UVParseState::ExtraParam => {
                        tag.self_closing = true;
                        tag.span.end = token.span.end;
                        return Ok(tag);
                    },
                    _ => {
                        return Err(SpannedError::new("Unexpected `/>` token", token.span));
                    },
                },
                UVLexerTokens::OpeningAngleBracketSlash => match parse_state {
                    UVParseState::TagBody => parse_state = UVParseState::ClosingTagName,
                    _ => {
                        return Err(SpannedError::new("Unexpected `</` token", token.span));
                    },
                },
                UVLexerTokens::Literal(lit) | UVLexerTokens::RawString(lit) => match parse_state {
                    UVParseState::TagName => {
                        tag.name = lit.to_owned();
                        parse_state = UVParseState::ExtraParam;
                    },
                    UVParseState::ExtraParam => {
                        parse_state = UVParseState::ClosingAngleBracketOpeningTag;
                        tag.extra_param = lit.to_owned();
                    },

                    UVParseState::TagBody => {
                        tag.children.push(UVParseBody::String(Spanned {
                            value: lit.to_owned(),
                            span: Span::new(token.span.start, token.span.end),
                        }));
                    },
                    UVParseState::ClosingTagName => {
                        parse_state = UVParseState::ClosingAngleBracketClosingTag;
                        closing_tag_name = lit.to_owned();
                    },
                    _ => {
                        return Err(SpannedError::new(format!("Unexpected literal `{}`", lit), token.span));
                    },
                },
                UVLexerTokens::Unknown(ch) => {
                    return Err(SpannedError::new(format!("Unexpected token: `{}`", ch), token.span));
                },
            }
        }

        let span = match self.iter.vec.last() {
            Some(token) => Span::new(token.span.end - 3, token.span.end),
            None => Span::default(),
        };
        Err(SpannedError::new("Unexpected EOF", span))
    }
}

#[cfg(test)]
mod tests {
    use ultraviolet_core::types::frontend::{
        Span, Spanned,
        tokens::{UVParseBody, UVParseNode},
    };

    use crate::{lexer::Lexer, tokens_parser::TokenParser};

    fn get_nodes(code: &str) -> UVParseNode {
        TokenParser::new(Lexer::new(code.to_owned()).parse()).parse().unwrap()
    }

    #[test]
    fn simple() {
        assert_eq!(
            get_nodes("<main><inner/></main>"),
            UVParseNode {
                name: "main".to_owned(),
                children: vec![UVParseBody::Tag(Box::new(UVParseNode {
                    name: "inner".to_owned(),
                    children: vec![],
                    self_closing: true,
                    extra_param: String::new(),
                    span: Span::new(6, 14)
                }))],
                self_closing: false,
                extra_param: String::new(),
                span: Span::new(0, 21)
            }
        )
    }

    #[test]
    fn literal() {
        assert_eq!(
            get_nodes("<main>literal</main>"),
            UVParseNode {
                name: "main".to_owned(),
                children: vec![UVParseBody::String(Spanned {
                    value: "literal".to_owned(),
                    span: Span::new(6, 13)
                })],
                self_closing: false,
                extra_param: String::new(),
                span: Span::new(0, 20)
            }
        )
    }

    #[test]
    #[should_panic]
    fn unexpected_token() {
        get_nodes("<main>literal?</main>");
    }
}
