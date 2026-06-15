use std::{ops::Deref, rc::Rc};

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, FunctionCall, FunctionDefinition, FunctionDefinitionArg},
        tokens::UVParseNode,
    },
};

use crate::ast::{
    ASTParser, GeneratorOutputType, is_valid_identifier,
    type_parser::validate_and_parse_inner_type_block,
};

impl ASTParser {
    pub fn parse_function_definition(&self, node: &UVParseNode) -> GeneratorOutputType {
        let extra = node.search_extra_children(vec!["name", "arg", "returns", "body"]);

        if !extra.is_empty() {
            let first_extra = extra.first().unwrap_or_spanned(node.span)?;

            return Err(SpannedError::new(
                "Found extra children inside function definition",
                first_extra.get_span(),
            ));
        }

        // ---------------------------- Name ----------------------------

        let name = match node.get_one_tag_by_name("name") {
            Some(name_block) => {
                if name_block.children_len() != 1 || !name_block.all_literals() {
                    return Err(SpannedError::new("Invalid function name", name_block.span));
                }

                let name = name_block
                    .get_inner_literal()
                    .unwrap_or_spanned(node.span)?;

                if !is_valid_identifier(name) {
                    return Err(SpannedError::new(
                        format!("`{}` is not a valid name for function", name.deref()),
                        name.span,
                    ));
                }

                Some(name)
            },
            None => None,
        };

        // -------------------------- Arguments -------------------------
        let arguments = Self::parse_arguments_definition(node.get_many_tags_by_name("arg"))?;

        // --------------------------- Body -----------------------------

        let body = match node.get_one_tag_by_name("body") {
            Some(x) => x,
            None => {
                return Err(SpannedError::new("Function must have a body", node.span));
            },
        };

        Ok(ASTBlockType::FunctionDefinition(Box::new(Spanned::new(
            FunctionDefinition {
                name: name.cloned(),
                arguments,
                return_type: validate_and_parse_inner_type_block(node, "returns")?,
                body: Rc::new(self.parse_children_vec(body)?),
            },
            node.span,
        ))))
    }

    /// Parse function definition arguments
    pub fn parse_arguments_definition(
        args: Vec<&UVParseNode>,
    ) -> Result<Vec<FunctionDefinitionArg>, SpannedError> {
        args.into_iter()
            .map(|arg| {
                // Name
                let name_block = arg.get_one_tag_by_name("name").ok_or(SpannedError::new(
                    "Argument definition should have an inner <name> tag",
                    arg.span,
                ))?;

                if name_block.children_len() != 1 || !name_block.all_literals() {
                    return Err(SpannedError::new("Invalid argument name", name_block.span));
                }

                let name = name_block.get_inner_literal().unwrap_or_spanned(arg.span)?;

                if !is_valid_identifier(name) {
                    return Err(SpannedError::new(
                        format!("`{}` is not a valid name for argument", name.deref()),
                        name.span,
                    ));
                }

                Ok(FunctionDefinitionArg {
                    name: name.clone(),
                    arg_type: validate_and_parse_inner_type_block(arg, "type")?.ok_or(
                        SpannedError::new(
                            "Argument definition should have an `type` tag",
                            arg.span,
                        ),
                    )?,
                    span: arg.span,
                })
            })
            .collect()
    }

    /// Parse function call block
    pub fn parse_function_call(&self, node: &UVParseNode) -> GeneratorOutputType {
        if node.extra_param.is_empty() {
            return Err(SpannedError::new(
                "Function call must have an function name",
                node.span,
            ));
        }

        if !is_valid_identifier(&node.extra_param) {
            return Err(SpannedError::new(
                format!(
                    "{} is not a valid identifier for function call",
                    node.extra_param
                ),
                node.span,
            ));
        }

        Ok(ASTBlockType::FunctionCall(Box::new(Spanned::new(
            FunctionCall {
                name: node.extra_param.clone(),
                args: self.parse_function_call_arguments(node.get_all_tags())?,
            },
            node.span,
        ))))
    }

    pub fn parse_function_call_arguments(
        &self,
        args: Vec<&UVParseNode>,
    ) -> Result<Vec<Spanned<ASTBlockType>>, SpannedError> {
        args.into_iter()
            .map(|arg| {
                self.generate_ast(arg)
                    .map(|ast| Spanned::new(ast, arg.span))
            })
            .collect()
    }
}
