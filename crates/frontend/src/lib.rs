use std::rc::Rc;

use ultraviolet_core::{
    errors::SpannedError,
    types::{
        Environment,
        builtins::DefineBuiltinsType,
        frontend::{SourceFile, SourceFileParsed, typechecker::UVTypeVariable},
    },
};

use crate::{
    ast::ASTParser, dead_code::analyze_dead_code_program, lexer::Lexer,
    module_resolver::resolve_modules, tokens_parser::TokenParser, typechecker::typecheck,
};

pub mod ast;
mod dead_code;
mod iterator;
mod lexer;
mod module_resolver;
mod tokens_parser;
mod typechecker;

/// Process a source file
pub fn process_file(
    source: Rc<SourceFile>,
    alias: impl Into<String>,
    is_mod: bool,
) -> Result<SourceFileParsed, SpannedError> {
    let mut lexer = Lexer::new(source.clone());
    let tokens = lexer.parse();

    let mut token_parser = TokenParser::new(tokens, source.clone());
    let parse_tree = token_parser.parse()?;

    let ast_parser = ASTParser::new(parse_tree);

    let (ast, modules) = if is_mod {
        ast_parser.gen_module_ast()?
    } else {
        ast_parser.gen_main_ast()?
    };

    let modules = resolve_modules(&modules)?;

    let dead_code = analyze_dead_code_program(&ast);

    if !dead_code.is_empty() {
        dead_code.into_iter().for_each(|e| println!("{e}"));
    }

    let env = Environment::<UVTypeVariable>::new();
    env.define_builtins();
    typecheck(&ast, env)?;

    Ok(SourceFileParsed {
        source_file: source,
        ast,
        modules,
        alias: alias.into(),
    })
}
