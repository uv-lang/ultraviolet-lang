use ultraviolet_core::{
    errors::{SpannedError, error_renderer::ErrorRenderer},
    types::{
        Environment,
        builtins::DefineBuiltinsType,
        frontend::{SourceFile, ast::ASTBlockType, typechecker::UVTypeVariable},
    },
};

use crate::{
    ast::gen_main_ast, dead_code::analyze_dead_code_program, lexer::Lexer,
    tokens_parser::TokenParser, typechecker::typecheck,
};

pub mod ast;
mod dead_code;
mod iterator;
mod lexer;
mod tokens_parser;
mod typechecker;

pub fn process(source: &SourceFile) -> Result<ASTBlockType, SpannedError> {
    let mut lexer = Lexer::new(source.code.clone());
    let tokens = lexer.parse();

    let mut token_parser = TokenParser::new(tokens);
    let parse_tree = token_parser.parse()?;

    let ast = gen_main_ast(&parse_tree)?;
    let dead_code = analyze_dead_code_program(&ast);

    if !dead_code.is_empty() {
        dead_code
            .into_iter()
            .for_each(|e| println!("{}", e.display_with_source(source)));
    }

    let env = Environment::<UVTypeVariable>::new();
    env.define_builtins();
    typecheck(&ast, env)?;

    Ok(ast)
}
