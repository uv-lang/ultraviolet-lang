use std::cell::RefCell;

use anyhow::Result;
use regex::Regex;
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{
        Positional,
        ast::{StringToUVCompareOp, StringToUVLogicalOp, StringToUVMathOp, StringToUVType},
        token_parser::UnwrapOptionError,
    },
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, ModuleImport},
        tokens::UVParseNode,
    },
};

use once_cell::sync::Lazy;

mod compare_op;
mod conditional_op;
mod ffi;
mod functions;
mod logical_op;
mod loops;
mod math_op;
mod modules;
mod ops;
mod type_parser;
mod values;
mod variables;

pub type GeneratorOutputType = Result<ASTBlockType, SpannedError>;

static IDENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z_.][a-zA-Z0-9_.]*$").unwrap());

/// Check if provided string is a valid var/fn identifier
fn is_valid_identifier(s: &str) -> bool {
    IDENT_REGEX.is_match(s)
}

pub struct ASTParser {
    pub modules: RefCell<Vec<Spanned<ModuleImport>>>,
    pub nodes: UVParseNode,
}

impl ASTParser {
    pub fn new(node: UVParseNode) -> Self {
        Self {
            modules: Default::default(),
            nodes: node,
        }
    }
    /// Parse `main` content
    pub fn gen_main_ast(self) -> Result<(ASTBlockType, Vec<Spanned<ModuleImport>>), SpannedError> {
        if self.nodes.name.ne("main") {
            return Err(SpannedError::new(
                "The program must begin with the <main> tag",
                self.nodes.get_span(),
            ));
        }

        Ok((
            ASTBlockType::CodeBlock(Spanned::new(
                self.parse_children_vec(&self.nodes)?,
                self.nodes.get_span(),
            )),
            self.modules.into_inner(),
        ))
    }

    /// Parse `mod` content
    pub fn gen_module_ast(
        self,
    ) -> Result<(ASTBlockType, Vec<Spanned<ModuleImport>>), SpannedError> {
        if self.nodes.name.ne("mod") {
            return Err(SpannedError::new(
                "The module must begin with the <module> tag",
                self.nodes.get_span(),
            ));
        }

        Ok((
            ASTBlockType::ModuleBlock(Spanned::new(
                self.parse_children_vec(&self.nodes)?,
                self.nodes.get_span(),
            )),
            self.modules.into_inner(),
        ))
    }

    /// Main recursively invoked parsing function
    pub fn generate_ast(&self, node: &UVParseNode) -> GeneratorOutputType {
        Ok(match node.name.as_str() {
            // Parse variable declaration
            "let" if !node.self_closing => self.parse_var_definition(node)?,

            // Parse for loop declaration
            "for" if !node.self_closing => self.parse_for_loop(node)?,

            // Parse while loop declaration
            "while" if !node.self_closing => self.parse_while_loop(node)?,

            // Parse conditional operator
            "if" if !node.self_closing => self.parse_conditional_op(node)?,

            // Parse group block
            "g" if !node.self_closing => ASTBlockType::GroupBlock(Spanned::new(
                self.parse_children_vec(node)?,
                node.get_span(),
            )),

            // Parse return block
            "return" => self.parse_return(node)?,

            // Parse break
            "break" if node.self_closing => ASTBlockType::Break(Spanned::new((), node.get_span())),

            // Parse continue
            "continue" if node.self_closing => {
                ASTBlockType::Continue(Spanned::new((), node.get_span()))
            },

            // Parse function definition
            "fn" if !node.self_closing => self.parse_function_definition(node)?,

            // Parse function call
            "call" => self.parse_function_call(node)?,

            // Parse function call with trailing `$` symbol
            c if c.ends_with("$") => {
                let mut new_node = node.clone();
                new_node.extra_param = node.name.trim_end_matches("$").to_owned();
                self.parse_function_call(&new_node)?
            },

            // Parse modules import
            "import" if !node.self_closing => self.parse_module_import(node)?,

            // Parse module exports
            "export" if !node.self_closing => self.parse_export(node)?,

            // Parse ffi definition
            "ffi" if !node.self_closing => self.parse_ffi_definition(node)?,

            // Values such as int, float, etc.
            name if name.to_uvtype().is_some() => self.parse_value(node)?,

            // Parse math operations, such as sum, div, etc.
            name if name.to_uvmath().is_some() && !node.self_closing => self.parse_math_op(node)?,

            // Parse compare operators, such as eq, neq, etc.
            name if name.to_uvcompare().is_some() && !node.self_closing => {
                self.parse_compare_op(node)?
            },

            // Parse logical operators, such as and, or, not
            name if name.to_uvlogical().is_some() && !node.self_closing => {
                self.parse_logical_op(node)?
            },

            // Parse variable assign
            _ if !node.self_closing => self.parse_var_assign(node)?,

            // Parse variable access
            _ if node.self_closing => self.parse_var_access(node)?,

            name => {
                return Err(SpannedError::new(
                    format!("Unexpected `{name}` tag"),
                    node.get_span(),
                ));
            },
        })
    }

    /// Parse node children to ast
    pub fn parse_children_vec(
        &self,
        n: &UVParseNode,
    ) -> Result<Vec<Spanned<ASTBlockType>>, SpannedError> {
        if !n.all_tags() {
            let literal = n.get_inner_literal().unwrap_or_spanned(n.get_span())?;
            return Err(SpannedError::new("Unexpected literal", literal.get_span()));
        }

        n.get_all_tags()
            .iter()
            .map(|n| {
                self.generate_ast(n)
                    .map(|ast| Spanned::new(ast, n.get_span()))
            })
            .collect::<Result<Vec<Spanned<ASTBlockType>>, SpannedError>>()
    }

    /// Parse return block
    fn parse_return(&self, node: &UVParseNode) -> Result<ASTBlockType, SpannedError> {
        let ch = match node.get_tag_at(0) {
            Some(t) => Some(Box::new(self.generate_ast(t)?)),
            None => None,
        };

        Ok(ASTBlockType::Return(Spanned::new(ch, node.get_span())))
    }
}
