use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, ForLoop, WhileLoop},
        tokens::UVParseNode,
    },
};

use crate::ast::{ASTParser, GeneratorOutputType};

impl ASTParser {
    /// Parse for loop
    pub fn parse_for_loop(&self, node: &UVParseNode) -> GeneratorOutputType {
        let extra = node.search_extra_children(vec!["iter", "start", "end", "step", "body"]);

        if !extra.is_empty() {
            let first_extra = extra.first().unwrap_or_spanned(node.get_span())?;

            return Err(SpannedError::new(
                "Found extra children inside `for` loop declaration",
                first_extra.get_span(),
            ));
        }

        // Iterator
        let iterator_node = match node.get_one_tag_by_name("iter") {
            Some(x) if x.children_len() != 1 || !x.all_literals() => {
                return Err(SpannedError::new(
                    "`iter` child must have only one inner literal",
                    x.get_span(),
                ));
            },
            Some(x) => x,
            None => {
                return Err(SpannedError::new(
                    "`for` loop must have an `iter` child",
                    node.get_span(),
                ));
            },
        };

        let iterator = iterator_node
            .get_inner_literal()
            .unwrap_or_spanned(iterator_node.get_span())?;

        // Step
        let step = match node.get_one_tag_by_name("step") {
            Some(n) => Some(Spanned::new(
                self.generate_ast(Self::get_and_validate_inner_tag(node, "step")?)?,
                n.get_span(),
            )),
            None => None,
        };

        // Body
        let body = match node.get_one_tag_by_name("body") {
            Some(x) => x,
            None => {
                return Err(SpannedError::new(
                    "`for` loop must have a body",
                    node.get_span(),
                ));
            },
        };

        let start = Self::get_and_validate_inner_tag(node, "start")?;
        let end = Self::get_and_validate_inner_tag(node, "end")?;

        Ok(ASTBlockType::ForLoop(Box::new(Spanned::new(
            ForLoop {
                iterator: iterator.clone(),
                start: Spanned::new(self.generate_ast(start)?, start.get_span()),
                end: Spanned::new(self.generate_ast(end)?, end.get_span()),
                step,
                body: Spanned::new(self.parse_children_vec(body)?, body.get_span()),
            },
            node.get_span(),
        ))))
    }

    /// Parse while loop to ast
    pub fn parse_while_loop(&self, node: &UVParseNode) -> GeneratorOutputType {
        let extra = node.search_extra_children(vec!["test", "body"]);

        if !extra.is_empty() {
            let first_extra = extra.first().unwrap_or_spanned(node.get_span())?;

            return Err(SpannedError::new(
                "Found extra children inside `while` loop declaration",
                first_extra.get_span(),
            ));
        }

        // Body
        let body = match node.get_one_tag_by_name("body") {
            Some(x) => x,
            None => {
                return Err(SpannedError::new(
                    "`while` loop must have a body",
                    node.get_span(),
                ));
            },
        };

        let test = Self::get_and_validate_inner_tag(node, "test")?;

        Ok(ASTBlockType::WhileLoop(Spanned::new(
            Box::new(WhileLoop {
                test: Spanned::new(self.generate_ast(test)?, test.get_span()),
                body: Spanned::new(self.parse_children_vec(body)?, body.get_span()),
            }),
            node.get_span(),
        )))
    }

    /// Get inner tag by nme and validate its children
    fn get_and_validate_inner_tag<'a>(
        node: &'a UVParseNode,
        name: &'a str,
    ) -> Result<&'a UVParseNode, SpannedError> {
        let x_node = match node.get_one_tag_by_name(name) {
            Some(x) if x.children_len() != 1 || !x.all_tags() => {
                return Err(SpannedError::new(
                    format!("`{name}` child must have only one inner tag"),
                    x.get_span(),
                ));
            },
            Some(x) => x,
            None => {
                return Err(SpannedError::new(
                    format!("Loop must have an `{name}` tag"),
                    node.get_span(),
                ));
            },
        };

        x_node.get_tag_at(0).unwrap_or_spanned(x_node.get_span())
    }
}
