use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::{ArgumentsCount, StringToUVCompareOp},
    types::frontend::{
        ast::{ASTBlockType, CompareOp, CompareOpType},
        tokens::UVParseNode,
    },
};

use crate::ast::{GeneratorOutputType, parse_children_vec};

/// Parse Ultraviolet compare operators
pub fn parse_compare_op(node: &UVParseNode) -> GeneratorOutputType {
    let op_type = node
        .name
        .to_uvcompare()
        .ok_or(SpannedError::new("Unknown comparison operation", node.span))?;

    let children = parse_arguments(node, &op_type)?;

    Ok(ASTBlockType::CompareOp(CompareOp {
        op_type,
        operands: children,
        span: node.span,
    }))
}

/// Parse arguments for compare
fn parse_arguments(
    node: &UVParseNode,
    op_type: &CompareOpType,
) -> Result<Vec<ASTBlockType>, SpannedError> {
    if !node.all_tags() {
        return Err(SpannedError::new(
            "Unexpected literals inside comparison operation",
            node.span,
        ));
    }

    if node.children_len() < op_type.min_arguments_count() {
        return Err(SpannedError::new(
            format!(
                "Comparison operator cannot have less than {} operands",
                op_type.min_arguments_count()
            ),
            node.span,
        ));
    }

    if let Some(max_args) = op_type.max_arguments_count()
        && node.children_len() > max_args
    {
        return Err(SpannedError::new(
            format!(
                "Too many arguments for `{}` comparison operation",
                node.name
            ),
            node.span,
        ));
    }

    parse_children_vec(node)
}
