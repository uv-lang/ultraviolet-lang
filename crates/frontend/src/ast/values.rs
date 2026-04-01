use std::ops::Deref;

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::token_parser::UnwrapOptionError,
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, UVValue},
        tokens::UVParseNode,
    },
};

use crate::ast::GeneratorOutputType;

/// Parse UVValues.
/// Caller must guarantee, that tag name is one of data types!
pub fn parse_value(node: &UVParseNode) -> GeneratorOutputType {
    Ok(ASTBlockType::Value(Spanned::new(
        match node.name.as_str() {
            "int" => UVValue::Int(parse_int(node)?),
            "float" => UVValue::Float(parse_float(node)?),
            "str" => UVValue::String(parse_str(node)),
            "bool" => UVValue::Boolean(parse_boolean(node)?),
            "null" => {
                validate_null(node)?;
                UVValue::Null
            },
            "void" => {
                validate_null(node)?;
                UVValue::Void
            },
            _ => {
                return Err(SpannedError::new(
                    format!("Unknown value type `{}`", node.name),
                    node.span,
                ));
            },
        },
        node.span,
    )))
}

/// Guarantee, that node has only one child and this child is literal
fn validate_inner(node: &UVParseNode) -> Result<(), SpannedError> {
    if node.children_len() != 1 || !node.all_literals() {
        return Err(SpannedError::new(
            format!("Invalid value for `{}` type", node.name),
            node.span,
        ));
    }
    Ok(())
}

fn parse_int(node: &UVParseNode) -> Result<i64, SpannedError> {
    validate_inner(node)?;
    let inner_contents = node.get_inner_literal().unwrap_or_spanned(node.span)?;

    inner_contents.parse::<i64>().map_err(|_| {
        SpannedError::new(
            format!("Cannot parse `{}` to an integer", inner_contents.deref()),
            inner_contents.span,
        )
    })
}

fn parse_float(node: &UVParseNode) -> Result<f64, SpannedError> {
    validate_inner(node)?;
    let inner_contents = node.get_inner_literal().unwrap_or_spanned(node.span)?;

    inner_contents.parse::<f64>().map_err(|_| {
        SpannedError::new(
            format!("Cannot parse `{}` to a float", inner_contents.deref()),
            inner_contents.span,
        )
    })
}

fn parse_str(node: &UVParseNode) -> String {
    if let Some(lit) = node.get_inner_literal() {
        lit.deref().clone()
    } else {
        String::new()
    }
}

fn parse_boolean(node: &UVParseNode) -> Result<bool, SpannedError> {
    validate_inner(node)?;
    let inner_contents = node.get_inner_literal().unwrap_or_spanned(node.span)?;

    match inner_contents.as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(SpannedError::new(
            format!("Cannot parse `{}` to a boolean", inner_contents.deref()),
            inner_contents.span,
        )),
    }
}

fn validate_null(node: &UVParseNode) -> Result<(), SpannedError> {
    if !node.self_closing {
        return Err(SpannedError::new("That tag must be self-closing", node.span));
    }

    Ok(())
}
