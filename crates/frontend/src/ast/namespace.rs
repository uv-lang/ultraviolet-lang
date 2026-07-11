use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, Namespace},
        tokens::UVParseNode,
    },
};

use crate::ast::{ASTParser, GeneratorOutputType, is_valid_identifier};

impl ASTParser {
    /// Parse namespace block
    pub fn parse_namespace(&self, node: &UVParseNode) -> GeneratorOutputType {
        let name = node.extra_param.clone();

        if !is_valid_identifier(&name.value) {
            return Err(SpannedError::new(
                format!("{} is not a valid name for namespace", name),
                name.get_span(),
            ));
        }

        Ok(ASTBlockType::Namespace(Spanned::new(
            Namespace {
                name,
                body: self.parse_children_vec(node)?,
            },
            node.get_span(),
        )))
    }
}
