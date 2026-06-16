use std::rc::Rc;

use ultraviolet_core::{
    errors::SpannedError,
    types::frontend::{SourceFile, SourceFileParsed},
};

use crate::{
    ast::ASTParser, dead_code::analyze_dead_code_program, lexer::Lexer,
    module_resolver::resolve_modules, tokens_parser::TokenParser, typechecker::Typechecker,
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
    is_mod: bool,
) -> Result<Rc<SourceFileParsed>, SpannedError> {
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

    let modules = resolve_modules(source.clone(), &modules)?;
    let dead_code = analyze_dead_code_program(&ast);

    if !dead_code.is_empty() {
        dead_code.into_iter().for_each(|e| println!("{e}"));
    }

    let source_parsed = Rc::new(SourceFileParsed {
        ast,
        modules,
        source: source.clone(),
    });

    if !is_mod {
        let typechecker = Typechecker::new(source_parsed.clone(), "");
        typechecker.start_typecheck()?;
    }

    Ok(source_parsed)
}
