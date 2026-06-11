use crate::ast::{GeneratorOutputType, ops::parse_arguments};
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::StringToUVLogicalOp,
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, BuiltInOperation},
        tokens::UVParseNode,
    },
};

/// Parse Ultraviolet logical operators
pub fn parse_logical_op(node: &UVParseNode) -> GeneratorOutputType {
    let op_type = node
        .name
        .to_uvlogical()
        .ok_or(SpannedError::new("Unknown logical operation", node.span))?;

    let children = parse_arguments(node, &op_type)?;

    Ok(ASTBlockType::LogicalOp(Spanned::new(
        BuiltInOperation {
            op_type,
            operands: children,
        },
        node.span,
    )))
}
