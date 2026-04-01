use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, ForLoop, WhileLoop},
        tokens::UVParseNode,
    },
};

use crate::ast::{GeneratorOutputType, generate_ast, parse_children_vec};

/// Parse for loop
pub fn parse_for_loop(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["iterator", "start", "end", "step", "body"]);

    if !extra.is_empty() {
        let first_extra = extra.first().unwrap_or_spanned(node.span)?;

        return Err(SpannedError::new(
            "Found extra children inside `for` loop declaration",
            first_extra.get_span(),
        ));
    }

    // Iterator
    let iterator_node = match node.get_one_tag_by_name("iterator") {
        Some(x) if x.children_len() != 1 || !x.all_literals() => {
            return Err(SpannedError::new(
                "`iterator` child must have only one inner literal",
                x.span,
            ));
        },
        Some(x) => x,
        None => {
            return Err(SpannedError::new("`for` loop must have an `iterator` child", node.span));
        },
    };

    let iterator = iterator_node
        .get_inner_literal()
        .unwrap_or_spanned(iterator_node.span)?;

    // Step
    let step = match node.get_one_tag_by_name("step") {
        Some(_) => Some(generate_ast(get_and_validate_inner_tag(node, "step")?)?),
        None => None,
    };

    // Body
    let body = match node.get_one_tag_by_name("body") {
        Some(x) => x,
        None => return Err(SpannedError::new("`for` loop must have a body", node.span)),
    };

    Ok(ASTBlockType::ForLoop(Box::new(ForLoop {
        iterator: iterator.clone(),
        start: generate_ast(get_and_validate_inner_tag(node, "start")?)?,
        end: generate_ast(get_and_validate_inner_tag(node, "end")?)?,
        step,
        body: Spanned::new(parse_children_vec(body)?, body.span),
        span: node.span,
    })))
}

/// Parse while loop to ast
pub fn parse_while_loop(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["test", "body"]);

    if !extra.is_empty() {
        let first_extra = extra.first().unwrap_or_spanned(node.span)?;

        return Err(SpannedError::new(
            "Found extra children inside `while` loop declaration",
            first_extra.get_span(),
        ));
    }

    // Body
    let body = match node.get_one_tag_by_name("body") {
        Some(x) => x,
        None => {
            return Err(SpannedError::new("`while` loop must have a body", node.span));
        },
    };

    Ok(ASTBlockType::WhileLoop(Box::new(WhileLoop {
        test: generate_ast(get_and_validate_inner_tag(node, "test")?)?,
        body: Spanned::new(parse_children_vec(body)?, body.span),

        span: node.span,
    })))
}

/// Get inner tag by nme and validate its children
fn get_and_validate_inner_tag<'a>(node: &'a UVParseNode, name: &'a str) -> Result<&'a UVParseNode, SpannedError> {
    let x_node = match node.get_one_tag_by_name(name) {
        Some(x) if x.children_len() != 1 || !x.all_tags() => {
            return Err(SpannedError::new(
                format!("`{name}` child must have only one inner tag"),
                x.span,
            ));
        },
        Some(x) => x,
        None => {
            return Err(SpannedError::new(format!("Loop must have an `{name}` tag"), node.span));
        },
    };

    x_node.get_tag_at(0).unwrap_or_spanned(x_node.span)
}
