use std::path::PathBuf;

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, ModuleImport, VariableAccess},
        tokens::UVParseNode,
    },
};

use crate::ast::{ASTParser, GeneratorOutputType};

impl ASTParser {
    /// Parse module import tag <import></import>
    pub fn parse_module_import(&self, node: &UVParseNode) -> GeneratorOutputType {
        let extra = node.search_extra_children(vec!["path", "as"]);

        if !extra.is_empty() {
            let first_extra = extra.first().unwrap_or_spanned(node.get_span())?;

            return Err(SpannedError::new(
                "Found extra children inside module import",
                first_extra.get_span(),
            ));
        }

        let path_block = node.get_one_tag_by_name("path").ok_or(SpannedError::new(
            "Module import should have a path",
            node.get_span(),
        ))?;

        if !path_block.all_literals() {
            return Err(SpannedError::new(
                "Module import path should contain only literals",
                path_block.get_span(),
            ));
        }

        let path =
            path_block
                .get_all_literals()
                .into_iter()
                .fold(String::new(), |mut acc, curr| {
                    acc.push_str(&curr.value);
                    acc
                });

        let alias = match node.get_one_tag_by_name("as") {
            Some(n) if !n.all_literals() || n.children_len() != 1 => {
                return Err(SpannedError::new(
                    "Module import alias should contain literal",
                    path_block.get_span(),
                ));
            },
            Some(n) => Some(n.get_inner_literal().unwrap_or_spanned(node.get_span())?),
            None => None,
        };

        let name = if let Some(al) = alias {
            al.clone().unwrap()
        } else {
            PathBuf::from(path.clone())
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        };

        let module = Spanned::new(
            ModuleImport {
                path: Spanned::new(path, path_block.get_span()),
                name: Spanned::new(name, path_block.get_span()),
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

    /// Get module export block
    pub fn parse_export(&self, node: &UVParseNode) -> GeneratorOutputType {
        if !node.all_tags() {
            return Err(SpannedError::new(
                "All nodes inside `export` must be tags",
                node.get_span(),
            ));
        }

        let exports = node
            .get_all_tags()
            .iter()
            .map(|t| {
                if !t.self_closing {
                    return Err(SpannedError::new(
                        "Variable access block should be self-closing",
                        t.get_span(),
                    ));
                }

                Ok(Spanned::new(
                    VariableAccess {
                        name: t.name.clone(),
                    },
                    t.get_span(),
                ))
            })
            .collect::<Result<Vec<Spanned<VariableAccess>>, SpannedError>>()?;

        if exports.is_empty() {
            return Err(SpannedError::new(
                "Module cannot export empty space",
                node.get_span(),
            ));
        }

        Ok(ASTBlockType::ModuleExport(Spanned::new(
            exports,
            node.get_span(),
        )))
    }
}
