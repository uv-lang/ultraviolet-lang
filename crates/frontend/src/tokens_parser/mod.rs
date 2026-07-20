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
                UVLexerTokens::Literal(lit) | UVLexerTokens::RawString(lit) => match parse_state {
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
    use std::{path::Path, rc::Rc};

    use ultraviolet_core::types::frontend::{
        SourceFile, Span, Spanned,
        tokens::{UVParseBody, UVParseNode},
    };

    use crate::{lexer::Lexer, tokens_parser::TokenParser};

    fn get_source_file(code: &str) -> SourceFile {
        SourceFile::from_str(code, Box::<Path>::from(Path::new("")))
    }

    fn get_nodes(sf: Rc<SourceFile>) -> UVParseNode {
        let tokens = Lexer::new(sf.clone()).parse();

        TokenParser::new(tokens, sf.clone()).parse().unwrap()
    }

    #[test]
    fn simple() {
        let sf = Rc::new(get_source_file("<main><inner/></main>"));
        assert_eq!(
            get_nodes(sf.clone()),
            UVParseNode {
                name: Spanned::new("main".to_owned(), Span::new(1, 5, sf.clone())),
                children: vec![UVParseBody::Tag(Box::new(UVParseNode {
                    name: Spanned::new("inner".to_owned(), Span::new(7, 12, sf.clone())),
                    children: vec![],
                    self_closing: true,
                    extra_param: Spanned::new(String::new(), Span::new(0, 0, sf.clone())),
                    span: Span::new(6, 14, sf.clone())
                }))],
                self_closing: false,
                extra_param: Spanned::new(String::new(), Span::new(0, 0, sf.clone())),
                span: Span::new(0, 21, sf.clone())
            }
        )
    }

    #[test]
    fn literal() {
        let sf = Rc::new(get_source_file("<main>literal</main>"));
        assert_eq!(
            get_nodes(sf.clone()),
            UVParseNode {
                name: Spanned::new("main".to_owned(), Span::new(1, 5, sf.clone())),
                children: vec![UVParseBody::String(Spanned {
                    value: "literal".to_owned(),
                    span: Span::new(6, 13, sf.clone())
                })],
                self_closing: false,
                extra_param: Spanned::new(String::new(), Span::new(0, 0, sf.clone())),
                span: Span::new(0, 20, sf.clone())
            }
        )
    }

    #[test]
    #[should_panic]
    fn unexpected_token() {
        get_nodes(Rc::new(get_source_file("<main>literal?</main>")));
    }
}
