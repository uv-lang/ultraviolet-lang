use std::{ops::Deref, rc::Rc};

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::frontend::{
        SourceFile, Span, Spanned,
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
    source_file: Rc<SourceFile>,
}

impl TokenParser {
    /// Create new TokenParser and pass tokens
    pub fn new(tokens: Vec<UVToken>, sf: Rc<SourceFile>) -> Self {
        Self {
            iter: Iter::from(tokens),
            source_file: sf,
        }
    }

    /// Parse and get Parse Tree
    pub fn parse(&mut self) -> Result<UVParseNode, SpannedError> {
        let mut parse_state = UVParseState::Unknown;
        let mut tag = {
            let span = Span::new(0, 0, self.source_file.clone());
            UVParseNode {
                name: Spanned::new(String::default(), span.clone()),
                children: Vec::new(),
                self_closing: false,
                extra_param: Spanned::new(String::default(), span.clone()),
                span,
            }
        };

        let mut closing_tag_name = String::new();
        while let Some(token) = self.iter.next() {
            match &token.token {
                UVLexerTokens::OpeningAngleBracket => match parse_state {
                    UVParseState::Unknown => {
                        parse_state = UVParseState::TagName;
                        tag.span.start = token.get_span().start;
                    },
                    UVParseState::TagBody => {
                        self.iter.step_back();
                        tag.children.push(UVParseBody::Tag(Box::new(self.parse()?)));
                    },
                    _ => {
                        return Err(SpannedError::new("Unexpected `<` token", token.get_span()));
                    },
                },
                UVLexerTokens::ClosingAngleBracket => match parse_state {
                    UVParseState::ClosingAngleBracketOpeningTag | UVParseState::ExtraParam => {
                        parse_state = UVParseState::TagBody
                    },
                    UVParseState::ClosingAngleBracketClosingTag => {
                        if tag.name.value.ne(&closing_tag_name) {
                            return Err(SpannedError::new(
                                format!(
                                    "Unexpected closing tag `{}`. Expected `{}`",
                                    closing_tag_name, tag.name
                                ),
                                Span::new(
                                    token.get_span().start - closing_tag_name.len(),
                                    token.get_span().end - 1,
                                    self.source_file.clone(),
                                ),
                            ));
                        }

                        tag.span.end = token.span.end;
                        return Ok(tag);
                    },
                    _ => {
                        return Err(SpannedError::new("Unexpected `>` token", token.get_span()));
                    },
                },
                UVLexerTokens::SelfClosingAngleBracket => match parse_state {
                    UVParseState::ClosingAngleBracketOpeningTag | UVParseState::ExtraParam => {
                        tag.self_closing = true;
                        tag.span.end = token.span.end;
                        return Ok(tag);
                    },
                    _ => {
                        return Err(SpannedError::new("Unexpected `/>` token", token.get_span()));
                    },
                },
                UVLexerTokens::OpeningAngleBracketSlash => match parse_state {
                    UVParseState::TagBody => parse_state = UVParseState::ClosingTagName,
                    _ => {
                        return Err(SpannedError::new("Unexpected `</` token", token.get_span()));
                    },
                },
                UVLexerTokens::Literal(lit) | UVLexerTokens::RawString(lit) => {
                    match parse_state {
                        UVParseState::TagName => {
                            tag.name = Spanned::new(lit.to_owned(), token.get_span());
                            parse_state = UVParseState::ExtraParam;
                        },
                        UVParseState::ExtraParam => {
                            parse_state = UVParseState::ClosingAngleBracketOpeningTag;
                            tag.extra_param = Spanned::new(lit.to_owned(), token.get_span());
                        },

                        UVParseState::TagBody => {
                            tag.children.push(UVParseBody::String(Spanned::new(
                                lit.to_owned(),
                                token.get_span(),
                            )));
                        },
                        UVParseState::ClosingTagName => {
                            parse_state = UVParseState::ClosingAngleBracketClosingTag;
                            closing_tag_name = lit.to_owned();
                        },
                        _ => {
                            return Err(SpannedError::new(
                                format!("Unexpected literal `{lit}`"),
                                token.get_span(),
                            ));
                        },
                    }
                },
                UVLexerTokens::Unknown(ch) => {
                    if matches!(parse_state, UVParseState::TagBody)
                        && tag.name.deref().eq("path")
                        && ch.eq(&'/')
                    {
                        tag.children.push(UVParseBody::String(Spanned::new(
                            String::from(*ch),
                            token.span,
                        )));
                    } else {
                        return Err(SpannedError::new(
                            format!("Unexpected token: `{ch}`"),
                            token.get_span(),
                        ));
                    }
                },
            }
        }

        let span = match self.iter.vec.last() {
            Some(token) => Span::new(
                token.get_span().end - 3,
                token.get_span().end,
                self.source_file.clone(),
            ),
            None => Span::new(0, 0, self.source_file.clone()),
        };
        Err(SpannedError::new("Unexpected EOF", span))
    }
}

#[cfg(test)]
mod tests {
    /*
    use ultraviolet_core::types::frontend::{
        Span, Spanned,
        tokens::{UVParseBody, UVParseNode},
    };

    use crate::{lexer::Lexer, tokens_parser::TokenParser};

    fn get_nodes(code: &str) -> UVParseNode {
        TokenParser::new(Lexer::new(code.to_owned()).parse())
            .parse()
            .unwrap()
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

    */
}
