use crate::ast::{ASTParser, GeneratorOutputType};
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::StringToUVMathOp},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, BuiltInOperation},
        tokens::UVParseNode,
    },
};

impl ASTParser {
    pub fn parse_math_op(&self, node: &UVParseNode) -> GeneratorOutputType {
        let op_type = node
            .name
            .to_uvmath()
            .ok_or(SpannedError::new("Unknown math operation", node.get_span()))?;

        let children = self.parse_arguments_for_operator(node, &op_type)?;

        Ok(ASTBlockType::MathOp(Spanned::new(
            BuiltInOperation {
                op_type,
                operands: children,
            },
            node.get_span(),
        )))
    }
}
