use crate::ast::{ASTParser, GeneratorOutputType};
use std::{error::Error, ops::Deref};
use ultraviolet_core::{
    errors::{CommonError, SpannedError},
    number_variants,
    traits::frontend::{Positional, ast::StringToUVNumberType, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, UVValue},
        number::Number,
        tokens::UVParseNode,
    },
};

impl ASTParser {
    /// Parse UVValues.
    /// Caller must guarantee, that tag name is one of data types!
    pub fn parse_value(&self, node: &UVParseNode) -> GeneratorOutputType {
        Ok(ASTBlockType::Value(Spanned::new(
            match node.name.as_str() {
                s if s.to_uv_number_type().is_some() => UVValue::Number(parse_number(node)?),

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
                        node.get_span(),
                    ));
                },
            },
            node.get_span(),
        )))
    }
}

/// Guarantee, that node has only one child and this child is literal
fn validate_inner(node: &UVParseNode) -> Result<(), SpannedError> {
    if node.children_len() != 1 || !node.all_literals() {
        return Err(SpannedError::new(
            format!("Invalid value for `{}` type", node.name),
            node.get_span(),
        ));
    }
    Ok(())
}

// Generate number parsing function for all number types
macro_rules! gen_parse_number_fn {
    ($($variant:ident($ty:ty,$a:ident)),* $(,)?) => {
        fn parse_number(node: &UVParseNode) -> Result<Number, SpannedError> {
            validate_inner(node)?;
            let inner_contents = node.get_inner_literal().unwrap_or_spanned(node.get_span())?;

            let parse = || -> Result<Number, Box<dyn Error>> {
                match node.name.as_str() {
                    $(stringify!($ty) => Ok(Number::$variant(inner_contents.parse::<$ty>()?)),)*
                    _ => Err(Box::new(CommonError::new("Unknown number type")))
                }
            };

            parse().map_err(|_| {
                SpannedError::new(
                    format!(
                        "Cannot parse `{}` to an `{}`",
                        inner_contents.deref(),
                        node.name
                    ),
                    inner_contents.get_span(),
                )
            })
        }
    };
}

number_variants!(gen_parse_number_fn);

fn parse_str(node: &UVParseNode) -> String {
    if let Some(lit) = node.get_inner_literal() {
        lit.deref().clone()
    } else {
        String::new()
    }
}

fn parse_boolean(node: &UVParseNode) -> Result<bool, SpannedError> {
    validate_inner(node)?;
    let inner_contents = node
        .get_inner_literal()
        .unwrap_or_spanned(node.get_span())?;

    match inner_contents.as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(SpannedError::new(
            format!("Cannot parse `{}` to a boolean", inner_contents.deref()),
            inner_contents.get_span(),
        )),
    }
}

fn validate_null(node: &UVParseNode) -> Result<(), SpannedError> {
    if !node.self_closing {
        return Err(SpannedError::new(
            "That tag must be self-closing",
            node.get_span(),
        ));
    }

    Ok(())
}
