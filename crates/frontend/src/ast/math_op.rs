use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::{ArgumentsCount, StringToUVMathOp},
    types::frontend::{
        ast::{ASTBlockType, MathOp, MathOpType},
        tokens::UVParseNode,
    },
};

use crate::ast::{GeneratorOutputType, parse_children_vec};

pub fn parse_math_op(node: &UVParseNode) -> GeneratorOutputType {
    let op_type = node
        .name
        .to_uvmath()
        .ok_or(SpannedError::new("Unknown math operation", node.span))?;

    let children = parse_arguments(node, &op_type)?;

    Ok(ASTBlockType::MathOp(MathOp {
        op_type,
        operands: children,
        span: node.span,
    }))
}

/// Parse arguments for math functions
pub fn parse_arguments(
    node: &UVParseNode,
    op_type: &MathOpType,
) -> Result<Vec<ASTBlockType>, SpannedError> {
    if !node.all_tags() {
        return Err(SpannedError::new(
            "Unexpected literals inside math operation",
            node.span,
        ));
    }

    if node.children_len() < op_type.min_arguments_count() {
        return Err(SpannedError::new(
            format!(
                "`{}` cannot have less than {} operands",
                node.name,
                op_type.min_arguments_count()
            ),
            node.span,
        ));
    }

    if let Some(max_args) = op_type.max_arguments_count()
        && node.children_len() > max_args
    {
        return Err(SpannedError::new(
            format!("Too much operands for `{}` math operation", node.name),
            node.span,
        ));
    }

    parse_children_vec(node)
}
