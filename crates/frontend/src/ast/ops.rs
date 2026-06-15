use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::ArgumentsCount,
    types::frontend::{Spanned, ast::ASTBlockType, tokens::UVParseNode},
};

use crate::ast::ASTParser;

impl ASTParser {
    /// Parse arguments operators
    pub fn parse_arguments_for_operator<T: ArgumentsCount>(
        &self,
        node: &UVParseNode,
        op_type: &T,
    ) -> Result<Vec<Spanned<ASTBlockType>>, SpannedError> {
        if !node.all_tags() {
            return Err(SpannedError::new(
                "Unexpected literals inside this operation",
                node.span,
            ));
        }

        if node.children_len() < op_type.min_arguments_count() {
            return Err(SpannedError::new(
                format!(
                    "This operator cannot have less than {} operands",
                    op_type.min_arguments_count()
                ),
                node.span,
            ));
        }

        if let Some(max_args) = op_type.max_arguments_count()
            && node.children_len() > max_args
        {
            return Err(SpannedError::new(
                format!("Too many arguments for `{}` operation", node.name),
                node.span,
            ));
        }

        self.parse_children_vec(node)
    }
}
