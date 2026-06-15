use crate::ast::{ASTParser, GeneratorOutputType};
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::StringToUVLogicalOp},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, BuiltInOperation},
        tokens::UVParseNode,
    },
};

impl ASTParser {
    /// Parse Ultraviolet logical operators
    pub fn parse_logical_op(&self, node: &UVParseNode) -> GeneratorOutputType {
        let op_type = node.name.to_uvlogical().ok_or(SpannedError::new(
            "Unknown logical operation",
            node.get_span(),
        ))?;

        let children = self.parse_arguments_for_operator(node, &op_type)?;

        Ok(ASTBlockType::LogicalOp(Spanned::new(
            BuiltInOperation {
                op_type,
                operands: children,
            },
            node.get_span(),
        )))
    }
}
