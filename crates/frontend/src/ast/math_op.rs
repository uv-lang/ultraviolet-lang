use crate::ast::{GeneratorOutputType, ops::parse_arguments};
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::StringToUVMathOp,
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, BuiltInOperation},
        tokens::UVParseNode,
    },
};

pub fn parse_math_op(node: &UVParseNode) -> GeneratorOutputType {
    let op_type = node
        .name
        .to_uvmath()
        .ok_or(SpannedError::new("Unknown math operation", node.span))?;

    let children = parse_arguments(node, &op_type)?;

    Ok(ASTBlockType::MathOp(Spanned::new(
        BuiltInOperation {
            op_type,
            operands: children,
        },
        node.span,
    )))
}
