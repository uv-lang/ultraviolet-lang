use anyhow::Result;
use regex::Regex;
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{
        ast::{StringToUVCompareOp, StringToUVLogicalOp, StringToUVMathOp, StringToUVType},
        token_parser::UnwrapOptionError,
    },
    types::frontend::{
        ast::{ASTBlockType, ProgramBlock},
        tokens::UVParseNode,
    },
};

use crate::ast::{
    compare_op::parse_compare_op,
    conditional_op::parse_conditional_op,
    functions::{parse_function_call, parse_function_definition},
    logical_op::parse_logical_op,
    loops::{parse_for_loop, parse_while_loop},
    math_op::parse_math_op,
    values::parse_value,
    variables::{parse_var_access, parse_var_assign, parse_var_definition},
};
use once_cell::sync::Lazy;

mod compare_op;
mod conditional_op;
mod functions;
mod logical_op;
mod loops;
mod math_op;
mod type_parser;
mod values;
mod variables;

pub type GeneratorOutputType = Result<ASTBlockType, SpannedError>;

static IDENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z_.][a-zA-Z0-9_.]*$").unwrap());

/// Check if provided string is a valid var/fn identifier
fn is_valid_identifier(s: &str) -> bool {
    IDENT_REGEX.is_match(s)
}

/// Parse `program` content
pub fn gen_main_ast(node: &UVParseNode) -> GeneratorOutputType {
    if node.name.ne("program") {
        return Err(SpannedError::new(
            "The program must begin with the <program> tag",
            node.span,
        ));
    }

    let head_parsed = if let Some(h) = node.get_one_tag_by_name("head") {
        Some(ASTBlockType::HeadBlock(parse_children_vec(h)?))
    } else {
        None
    };

    let main = ASTBlockType::MainBlock(parse_children_vec(
        node.get_one_tag_by_name("main")
            .ok_or(SpannedError::new("Main block in <program> is required", node.span))?,
    )?);

    Ok(ASTBlockType::Program(Box::new(ProgramBlock {
        head: head_parsed,
        main,
        span: node.span,
    })))
}

/// Main recursively invoked parsing function
pub fn generate_ast(node: &UVParseNode) -> GeneratorOutputType {
    Ok(match node.name.as_str() {
        // Parse variable declaration
        "let" if !node.self_closing => parse_var_definition(node)?,

        // Parse for loop declaration
        "for" if !node.self_closing => parse_for_loop(node)?,

        // Parse while loop declaration
        "while" if !node.self_closing => parse_while_loop(node)?,

        // Parse conditional operator
        "if" if !node.self_closing => parse_conditional_op(node)?,

        // Parse group block
        "g" if !node.self_closing => ASTBlockType::GroupBlock(Box::new(parse_children_vec(node)?)),

        // Parse return block
        // TODO: Dead code analysis
        "return" if !node.self_closing => parse_return(node)?,

        // Parse function definition
        "fn" if !node.self_closing => parse_function_definition(node)?,

        // Parse function call
        "call" => parse_function_call(node)?,

        // Values such as int, float, etc.
        name if name.to_uvtype().is_some() => parse_value(node)?,

        // Parse math operations, such as sum, div, etc.
        name if name.to_uvmath().is_some() && !node.self_closing => parse_math_op(node)?,

        // Parse compare operators, such as eq, neq, etc.
        name if name.to_uvcompare().is_some() && !node.self_closing => parse_compare_op(node)?,

        // Parse logical operators, such as and, or, not
        name if name.to_uvlogical().is_some() && !node.self_closing => parse_logical_op(node)?,

        // Parse variable assign
        _ if !node.self_closing => parse_var_assign(node)?,

        // Parse variable access
        _ if node.self_closing => parse_var_access(node)?,

        name => {
            return Err(SpannedError::new(format!("Unexpected `{name}` tag"), node.span));
        },
    })
}

/// Parse node children to ast
pub fn parse_children_vec(n: &UVParseNode) -> Result<Vec<ASTBlockType>, SpannedError> {
    if !n.all_tags() {
        let literal = n.get_inner_literal().unwrap_or_spanned(n.span)?;
        return Err(SpannedError::new("Unexpected literal", literal.span));
    }

    n.get_all_tags()
        .iter()
        .map(|n| generate_ast(n))
        .collect::<Result<Vec<ASTBlockType>, SpannedError>>()
}

/// Parse return block
fn parse_return(node: &UVParseNode) -> Result<ASTBlockType, SpannedError> {
    if !node.all_tags() || node.children_len() != 1 {
        return Err(SpannedError::new_tipped(
            "`return` statement should have one inner tag",
            "Try using group `g` block",
            node.span,
        ));
    }

    Ok(ASTBlockType::Return(Box::new(generate_ast(
        node.get_tag_at(0).unwrap_or_spanned(node.span)?,
    )?)))
}
