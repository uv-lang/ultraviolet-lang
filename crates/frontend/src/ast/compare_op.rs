use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::StringToUVCompareOp,
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, BuiltInOperation},
        tokens::UVParseNode,
    },
};

use crate::ast::{ASTParser, GeneratorOutputType};

impl ASTParser {
    /// Parse Ultraviolet compare operators
    pub fn parse_compare_op(&self, node: &UVParseNode) -> GeneratorOutputType {
        let op_type = node
            .name
            .to_uvcompare()
            .ok_or(SpannedError::new("Unknown comparison operation", node.span))?;

        let operands = self.parse_arguments_for_operator(node, &op_type)?;

        Ok(ASTBlockType::CompareOp(Spanned::new(
            BuiltInOperation { op_type, operands },
            node.span,
        )))
    }
}
