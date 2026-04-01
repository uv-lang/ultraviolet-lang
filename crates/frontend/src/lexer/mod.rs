use std::char;

use ultraviolet_core::types::frontend::{
    Span,
    lexer::{LexerParseState, UVLexerTokens, UVToken},
};

use crate::iterator::Iter;

pub struct Lexer {
    iter: Iter<char>,

    buffer: String,
    parse_state: LexerParseState,

    token_start: usize,
}

impl Lexer {
    pub fn new(input_code: String) -> Lexer {
        Self {
            iter: Iter::from(input_code.chars()),
            buffer: String::new(),
            parse_state: LexerParseState::Default,
            token_start: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<UVToken> {
        let mut tokens: Vec<UVToken> = Vec::new();
        while self.iter.peek(None).is_some() {
            let iteration_buffer = match self.parse_state {
                LexerParseState::Default => self.lex_normal_mode(),
                LexerParseState::ParsingRawStringLiteral(_) => self.lex_raw_mode(),
            };

            tokens.extend(iteration_buffer);
        }

        // If literal is a raw – disable trimming
        let trim_end = matches!(self.parse_state, LexerParseState::Default);

        if let Some(lit) = self.finish_consuming_literal(trim_end) {
            let token = match self.parse_state {
                LexerParseState::Default => UVLexerTokens::Literal(lit),
                LexerParseState::ParsingRawStringLiteral(_) => UVLexerTokens::RawString(lit),
            };
            tokens.push(UVToken {
                token,
                span: Span::new(self.token_start, self.iter.pos),
            });
        }
        tokens
    }

    fn lex_normal_mode(&mut self) -> Vec<UVToken> {
        let ch = self.iter.next().unwrap(); // This unwrap is potentially unreachable
        let mut iteration_buffer = Vec::<UVToken>::new();

        match ch {
            '<' | '>' | '/' => {
                if let Some(str) = self.finish_consuming_literal(true) {
                    iteration_buffer.push(UVToken {
                        token: UVLexerTokens::Literal(str.clone()),
                        span: Span::new(self.token_start, self.iter.pos - 1),
                    })
                }

                match ch {
                    '<' => {
                        if self.check_comment_and_consume() {
                            return iteration_buffer;
                        }

                        self.token_start = self.iter.pos - 1;
                        if self.iter.peek(None) == Some('/') {
                            self.iter.next(); // Consume '/'
                            iteration_buffer.push(UVToken {
                                token: UVLexerTokens::OpeningAngleBracketSlash,
                                span: Span::new(self.token_start, self.iter.pos),
                            });
                        } else {
                            iteration_buffer.push(UVToken {
                                token: UVLexerTokens::OpeningAngleBracket,
                                span: Span::new(self.token_start, self.iter.pos),
                            });
                        }

                        if let Some(key) = self.check_opening_raw_str_tag() {
                            self.parse_state = LexerParseState::ParsingRawStringLiteral(key);
                            iteration_buffer.extend([
                                UVToken {
                                    token: UVLexerTokens::Literal("str".to_string()),
                                    span: Span::new(self.token_start + 1, self.iter.pos - 1),
                                },
                                UVToken {
                                    token: UVLexerTokens::ClosingAngleBracket,
                                    span: Span::new(self.iter.pos - 1, self.iter.pos),
                                },
                            ]);
                            self.token_start = self.iter.pos;
                        }
                    },
                    '>' => {
                        self.token_start = self.iter.pos - 1;
                        iteration_buffer.push(UVToken {
                            token: UVLexerTokens::ClosingAngleBracket,
                            span: Span::new(self.token_start, self.iter.pos),
                        });
                    },
                    '/' => {
                        self.token_start = self.iter.pos - 1;
                        if self.iter.peek(None) == Some('>') {
                            self.iter.next(); // Consume '>'
                            iteration_buffer.push(UVToken {
                                token: UVLexerTokens::SelfClosingAngleBracket,
                                span: Span::new(self.token_start, self.iter.pos),
                            });
                        } else {
                            iteration_buffer.push(UVToken {
                                token: UVLexerTokens::Unknown('/'),
                                span: Span::new(self.token_start, self.iter.pos),
                            });
                        }
                    },
                    _ => {},
                }
            },
            char if Self::is_valid_literal(char) => {
                if self.buffer.is_empty() {
                    self.token_start = self.iter.pos - 1;
                }
                self.buffer.push(char);
            },

            char if !Self::is_valid_literal(char) => {
                if let Some(str) = self.finish_consuming_literal(true) {
                    iteration_buffer.push(UVToken {
                        token: UVLexerTokens::Literal(str),
                        span: Span::new(self.token_start, self.iter.pos - 1),
                    });
                }

                if !char.is_whitespace() {
                    iteration_buffer.push(UVToken {
                        token: UVLexerTokens::Unknown(char),
                        span: Span::new(self.iter.pos - 1, self.iter.pos),
                    })
                }
            },

            _ => {},
        }

        iteration_buffer
    }

    fn lex_raw_mode(&mut self) -> Vec<UVToken> {
        let ch = self.iter.next().unwrap(); // This unwrap is potentially unreachable
        let mut iteration_buffer = Vec::<UVToken>::new();

        self.buffer.push(ch);
        let token_end = self.iter.pos;

        if ch == '<' && self.check_closing_raw_str_tag() {
            self.buffer.pop(); // Remove '<' from buffer
            if let Some(str) = self.finish_consuming_literal(false) {
                iteration_buffer.push(UVToken {
                    token: UVLexerTokens::RawString(str),
                    span: Span::new(self.token_start, token_end - 1),
                });
            }
            iteration_buffer.extend([
                UVToken {
                    token: UVLexerTokens::OpeningAngleBracketSlash,
                    span: Span::new(token_end - 1, token_end + 1),
                },
                UVToken {
                    token: UVLexerTokens::Literal("str".to_string()),
                    span: Span::new(token_end + 1, self.iter.pos - 1),
                },
                UVToken {
                    token: UVLexerTokens::ClosingAngleBracket,
                    span: Span::new(self.iter.pos - 1, self.iter.pos),
                },
            ]);
            self.parse_state = LexerParseState::Default;
        }

        iteration_buffer
    }

    /// Returns buffered literal
    fn finish_consuming_literal(&mut self, trim: bool) -> Option<String> {
        let text = if trim { self.buffer.trim() } else { &self.buffer };

        let token = if text.is_empty() { None } else { Some(text.to_owned()) };

        self.buffer.clear();
        token
    }

    /// Consume all symbols after <str- and before >
    fn consume_raw_str_label(&mut self) -> Option<String> {
        let mut buffer = String::new();
        while let Some(char) = self.iter.next() {
            if char == '>' {
                return Some(buffer);
            } else {
                buffer.push(char);
            }
        }
        None
    }

    /// Check if iterator currently reach <str-xx> tag
    fn check_opening_raw_str_tag(&mut self) -> Option<Option<String>> {
        let start_iter_pos = self.iter.pos;
        self.iter.step_back(); // For proper consuming '<'

        if self.iter.starts_with(&['<', 's', 't', 'r']) {
            self.iter.pos += 4;
            match self.iter.next() {
                Some('>') => return Some(None),
                Some('-') => return Some(self.consume_raw_str_label()),
                _ => {},
            }
        }

        self.iter.pos = start_iter_pos;
        None
    }

    /// Check if iterator currently reach </str-xx> tag
    fn check_closing_raw_str_tag(&mut self) -> bool {
        let start_iter_pos = self.iter.pos;
        self.iter.step_back(); // For proper consuming '<'

        if self.iter.starts_with(&['<', '/', 's', 't', 'r']) {
            self.iter.pos += 5;

            match self.iter.next() {
                Some('>') if matches!(self.parse_state, LexerParseState::ParsingRawStringLiteral(None)) => {
                    return true;
                },
                Some('-') => {
                    let label = self.consume_raw_str_label();
                    if let Some(label) = label
                        && let LexerParseState::ParsingRawStringLiteral(Some(start_label)) = &self.parse_state
                        && start_label.eq(&label)
                    {
                        return true;
                    }
                },
                _ => {},
            }
        }

        self.iter.pos = start_iter_pos;
        false
    }

    fn check_comment_and_consume(&mut self) -> bool {
        if !self.iter.starts_with(&['!', '-', '-']) {
            return false;
        }

        while self.iter.next().is_some() {
            if self.iter.starts_with(&['-', '-', '>']) {
                self.iter.pos += 3;
                return true;
            }
        }
        true
    }

    fn is_valid_literal(c: char) -> bool {
        c.is_alphanumeric() || matches!(c, '.' | ',' | '_')
    }
}

#[cfg(test)]
mod tests {
    use ultraviolet_core::types::frontend::{
        Span,
        lexer::{UVLexerTokens, UVToken},
    };

    use crate::lexer::Lexer;

    fn get_tokens(code: &str) -> Vec<UVLexerTokens> {
        Lexer::new(code.to_owned())
            .parse()
            .into_iter()
            .map(|t| t.token)
            .collect::<Vec<UVLexerTokens>>()
    }

    #[test]
    fn parse_simple() {
        assert_eq!(
            get_tokens("<main><test /></main>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("test".to_owned()),
                UVLexerTokens::SelfClosingAngleBracket,
                UVLexerTokens::OpeningAngleBracketSlash,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn parse_inner_literal() {
        assert_eq!(
            get_tokens("<main>test</main>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::Literal("test".to_owned()),
                UVLexerTokens::OpeningAngleBracketSlash,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(
            get_tokens("<main>?</main>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::Unknown('?'),
                UVLexerTokens::OpeningAngleBracketSlash,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn parse_comments() {
        assert_eq!(
            get_tokens("<main><!-- this is a comment! --></main>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::OpeningAngleBracketSlash,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn unclosed_comment() {
        assert_eq!(
            get_tokens("<main><!-- this is an unclosed comment!</main>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("main".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn parse_raw_str() {
        assert_eq!(
            get_tokens("<str> Random content <null /> </str>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("str".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::RawString(" Random content <null /> ".to_owned()),
                UVLexerTokens::OpeningAngleBracketSlash,
                UVLexerTokens::Literal("str".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn parse_labeled_str() {
        assert_eq!(
            get_tokens("<str-test> Random content <str-123></str-123> <null /> </str-test>"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("str".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::RawString(" Random content <str-123></str-123> <null /> ".to_owned()),
                UVLexerTokens::OpeningAngleBracketSlash,
                UVLexerTokens::Literal("str".to_owned()),
                UVLexerTokens::ClosingAngleBracket
            ]
        )
    }

    #[test]
    fn parse_broken_raw_str() {
        assert_eq!(
            get_tokens("<str> Random content <null /> </str"),
            [
                UVLexerTokens::OpeningAngleBracket,
                UVLexerTokens::Literal("str".to_owned()),
                UVLexerTokens::ClosingAngleBracket,
                UVLexerTokens::RawString(" Random content <null /> </str".to_owned())
            ]
        )
    }

    #[test]
    fn test_indexes() {
        assert_eq!(
            Lexer::new("<main>test</main>".to_owned()).parse(),
            [
                UVToken {
                    token: UVLexerTokens::OpeningAngleBracket,
                    span: Span::new(0, 1)
                },
                UVToken {
                    token: UVLexerTokens::Literal("main".to_owned()),
                    span: Span::new(1, 5)
                },
                UVToken {
                    token: UVLexerTokens::ClosingAngleBracket,
                    span: Span::new(5, 6)
                },
                UVToken {
                    token: UVLexerTokens::Literal("test".to_owned()),
                    span: Span::new(6, 10)
                },
                UVToken {
                    token: UVLexerTokens::OpeningAngleBracketSlash,
                    span: Span::new(10, 12)
                },
                UVToken {
                    token: UVLexerTokens::Literal("main".to_owned()),
                    span: Span::new(12, 16)
                },
                UVToken {
                    token: UVLexerTokens::ClosingAngleBracket,
                    span: Span::new(16, 17)
                },
            ]
        )
    }
    #[test]
    fn test_labeled_indexes() {
        assert_eq!(
            Lexer::new("<str-123>test</str-123>".to_owned()).parse(),
            [
                UVToken {
                    token: UVLexerTokens::OpeningAngleBracket,
                    span: Span::new(0, 1)
                },
                UVToken {
                    token: UVLexerTokens::Literal("str".to_string()),
                    span: Span::new(1, 8)
                },
                UVToken {
                    token: UVLexerTokens::ClosingAngleBracket,
                    span: Span::new(8, 9)
                },
                UVToken {
                    token: UVLexerTokens::RawString("test".to_string()),
                    span: Span::new(9, 13)
                },
                UVToken {
                    token: UVLexerTokens::OpeningAngleBracketSlash,
                    span: Span::new(13, 15)
                },
                UVToken {
                    token: UVLexerTokens::Literal("str".to_string()),
                    span: Span::new(15, 22)
                },
                UVToken {
                    token: UVLexerTokens::ClosingAngleBracket,
                    span: Span::new(22, 23)
                },
            ]
        )
    }
}
