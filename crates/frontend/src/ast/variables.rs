use std::ops::Deref;

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, VariableAccess, VariableAssign, VariableDefinition},
        tokens::UVParseNode,
    },
};

use crate::ast::{
    ASTParser, GeneratorOutputType, is_valid_identifier,
    type_parser::validate_and_parse_inner_type_block,
};

impl ASTParser {
    /// Parse definition of variables <let></let>
    pub fn parse_var_definition(&self, node: &UVParseNode) -> GeneratorOutputType {
        let extra = node.search_extra_children(vec!["name", "value", "const", "type"]);
        if !extra.is_empty() {
            let first = extra.first().unwrap_or_spanned(node.get_span())?;
            return Err(SpannedError::new(
                "Found extra children for variable definition",
                first.get_span(),
            ));
        }

        let name_block = node.get_one_tag_by_name("name").ok_or(SpannedError::new(
            "Variable definition should have an inner <name> tag",
            node.get_span(),
        ))?;

        if name_block.children_len() != 1 || !name_block.all_literals() {
            return Err(SpannedError::new(
                "Invalid variable name",
                name_block.get_span(),
            ));
        }

        let name = name_block
            .get_inner_literal()
            .unwrap_or_spanned(node.get_span())?;

        if !is_valid_identifier(name) {
            return Err(SpannedError::new(
                format!("`{}` is not a valid name for variable", name.deref()),
                name.get_span(),
            ));
        }

        let value_block = node.get_one_tag_by_name("value").ok_or(SpannedError::new(
            "Variable must be initialized",
            node.get_span(),
        ))?;

        if value_block.children_len() != 1 || !value_block.all_tags() {
            return Err(SpannedError::new_tipped(
                "Variable value must have only one inner tag.",
                "If you want to place multiple tags, wrap them in a <g> tag.",
                value_block.get_span(),
            ));
        }

        let value = value_block
            .get_tag_at(0)
            .unwrap_or_spanned(node.get_span())?;

        // <const /> tag
        let is_const = match node.get_one_tag_by_name("const") {
            Some(c) if !c.self_closing => {
                return Err(SpannedError::new(
                    "`const` tag must be self-closing",
                    c.get_span(),
                ));
            },
            Some(_) => true,
            None => false,
        };

        Ok(ASTBlockType::VariableDefinition(Box::new(Spanned::new(
            VariableDefinition {
                name: Spanned::new(name.deref().clone(), name_block.get_span()),
                value: Spanned::new(self.generate_ast(value)?, value_block.get_span()),
                expected_type: validate_and_parse_inner_type_block(node, "type")?,
                is_const,
            },
            node.get_span(),
        ))))
    }

    /// Parse variable assignment
    pub fn parse_var_assign(&self, node: &UVParseNode) -> GeneratorOutputType {
        if !node.all_tags() {
            let unexpected_lit = node
                .get_inner_literal()
                .unwrap_or_spanned(node.get_span())?;

            return Err(SpannedError::new(
                "Cannot assign literal to a variable",
                unexpected_lit.get_span(),
            ));
        }

        if node.children_len() != 1 {
            let extra = node.get_child_at(1);

            return Err(SpannedError::new(
                "Variable assign should have only one nested tag",
                match extra {
                    Some(x) => x.get_span(),
                    None => node.get_span(),
                },
            ));
        }

        let value = node
            .get_tag_at(0)
            .ok_or(SpannedError::new("Cannot get inner tag", node.get_span()))?;

        Ok(ASTBlockType::VariableAssignment(Box::new(Spanned::new(
            VariableAssign {
                name: node.name.clone(),
                value: Spanned::new(self.generate_ast(value)?, value.get_span()),
            },
            node.get_span(),
        ))))
    }

    /// Parse variable access block
    pub fn parse_var_access(&self, node: &UVParseNode) -> GeneratorOutputType {
        if !node.self_closing {
            return Err(SpannedError::new(
                "Variable access block should be self-closing",
                node.get_span(),
            ));
        }


        // If variable is accessed as reference - create reference block
        if node.extra_param.eq("ref") {
            return Ok(ASTBlockType::ReferenceCreate(Spanned::new(
                VariableAccess {
                    name: node.name.clone(),
                },
                node.get_span(),
            )));
        }

        Ok(ASTBlockType::VariableAccess(Spanned::new(
            VariableAccess {
                name: node.name.clone(),
            },
            node.get_span(),
        )))
    }
}
