use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, ModuleImport},
        tokens::UVParseNode,
    },
};

use crate::ast::{ASTParser, GeneratorOutputType};

impl ASTParser {
    /// Parse module import tag <import></import>
    pub fn parse_module_import(&self, node: &UVParseNode) -> GeneratorOutputType {
        let extra = node.search_extra_children(vec!["name", "as"]);

        if !extra.is_empty() {
            let first_extra = extra.first().unwrap_or_spanned(node.get_span())?;

            return Err(SpannedError::new(
                "Found extra children inside module import",
                first_extra.get_span(),
            ));
        }

        let name_block = node.get_one_tag_by_name("name").ok_or(SpannedError::new(
            "Module import should have a name",
            node.get_span(),
        ))?;

        if !name_block.all_literals() || name_block.children_len() != 1 {
            return Err(SpannedError::new(
                "Module import name should contain literal",
                name_block.get_span(),
            ));
        }

        let name = name_block
            .get_inner_literal()
            .unwrap_or_spanned(name_block.get_span())?;

        let alias = match node.get_one_tag_by_name("as") {
            Some(n) if !n.all_literals() || n.children_len() != 1 => {
                return Err(SpannedError::new(
                    "Module import alias should contain literal",
                    name_block.get_span(),
                ));
            },
            Some(n) => Some(n.get_inner_literal().unwrap_or_spanned(node.get_span())?),
            None => None,
        };

        let module = Spanned::new(
            ModuleImport {
                name: name.clone(),
                alias: alias.cloned(),
            },
            node.get_span(),
        );

        self.modules
            .try_borrow_mut()
            .map_err(|_| {
                SpannedError::new(
                    "[INTERNAL ERROR] Cannot acquire internal modules store",
                    node.get_span(),
                )
            })?
            .push(module.clone());

        Ok(ASTBlockType::ModuleImport(module))
    }
}
