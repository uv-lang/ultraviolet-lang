use ultraviolet_core::{
    errors::SpannedError,
    types::frontend::{SourceFile, ast::ASTBlockType},
};

use crate::{ast::gen_main_ast, lexer::Lexer, tokens_parser::TokenParser};

pub mod ast;
mod iterator;
mod lexer;
mod tokens_parser;

pub fn process(source: &SourceFile) -> Result<ASTBlockType, SpannedError> {
    let mut lexer = Lexer::new(source.code.clone());
    let tokens = lexer.parse();

    let mut token_parser = TokenParser::new(tokens);
    let parse_tree = token_parser.parse()?;

    gen_main_ast(&parse_tree)
}
