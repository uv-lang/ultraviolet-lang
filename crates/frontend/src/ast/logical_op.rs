use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::{ArgumentsCount, StringToUVLogicalOp},
    types::frontend::{
        ast::{ASTBlockType, LogicalOp},
        tokens::UVParseNode,
    },
};

use crate::ast::{GeneratorOutputType, parse_children_vec};

/// Parse Ultraviolet logical operators
pub fn parse_logical_op(node: &UVParseNode) -> GeneratorOutputType {
    let op_type = node
        .name
        .to_uvlogical()
        .ok_or(SpannedError::new("Unknown logical operation", node.span))?;

    let children = parse_arguments(
        node,
        op_type.min_arguments_count(),
        op_type.max_arguments_count(),
    )?;

    Ok(ASTBlockType::LogicalOp(LogicalOp {
        op_type,
        operands: children,
        span: node.span,
    }))
}

/// Parse arguments for logical op
fn parse_arguments(
    node: &UVParseNode,
    min: usize,
    max: Option<usize>,
) -> Result<Vec<ASTBlockType>, SpannedError> {
    if !node.all_tags() {
        return Err(SpannedError::new(
            "Unexpected literals inside logical operation",
            node.span,
        ));
    }

    if node.children_len() < min {
        return Err(SpannedError::new(
            format!("Comparison operator cannot have less than {min} operands"),
            node.span,
        ));
    }

    if let Some(m) = max
        && node.children_len() > m
    {
        return Err(SpannedError::new(
            format!(
                "`{}` logical operation can handle only {} arguments",
                node.name, m
            ),
            node.span,
        ));
    }

    parse_children_vec(node)
}
