use std::ops::Deref;

use crate::ast::{
    GeneratorOutputType, generate_ast, is_valid_identifier,
    type_parser::{parse_type_raw, validate_and_parse_inner_type_block},
};
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        ast::{ASTBlockType, FFIDefinition},
        tokens::UVParseNode,
        types::UVType,
    },
};

/// Parse <ffi> structure
pub fn parse_ffi_definition(node: &UVParseNode) -> GeneratorOutputType {
    let extra = node.search_extra_children(vec!["name", "dll", "func", "arg", "returns"]);

    if !extra.is_empty() {
        let first_extra = extra.first().unwrap_or_spanned(node.span)?;

        return Err(SpannedError::new(
            "Found extra children inside FFI definition",
            first_extra.get_span(),
        ));
    }

    // ------------------------- Name -------------------------

    let name = match node.get_one_tag_by_name("name") {
        Some(i) if i.children_len() != 1 || !i.all_literals() => {
            Err(SpannedError::new("Invalid FFI name", i.span))
        },
        Some(i) => {
            let n = i.get_inner_literal().unwrap_or_spanned(i.span)?;
            if !is_valid_identifier(n) {
                return Err(SpannedError::new(
                    format!("`{}` is not a valid name for FFI", n.deref()),
                    n.span,
                ));
            }

            Ok(n.clone())
        },
        None => Err(SpannedError::new(
            "FFI definition should have an inner <name> tag",
            node.span,
        )),
    }?;

    // ------------------------- dll -------------------------
    let dll = match node.get_one_tag_by_name("dll") {
        Some(i) if i.children_len() != 1 || !i.all_tags() => Err(SpannedError::new(
            "`dll` block should have only one inner tag",
            i.span,
        )),
        Some(i) => Ok(Spanned::new(
            generate_ast(i.get_tag_at(0).unwrap_or_spanned(i.span)?)?,
            i.span,
        )),
        None => Err(SpannedError::new(
            "FFI definition should have an inner <dll> tag",
            node.span,
        )),
    }?;

    // ------------------------- func -------------------------
    let func = match node.get_one_tag_by_name("func") {
        Some(i) if i.children_len() != 1 || !i.all_tags() => Err(SpannedError::new(
            "`func` block should have only one inner tag",
            i.span,
        )),
        Some(i) => Ok(Spanned::new(
            generate_ast(i.get_tag_at(0).unwrap_or_spanned(i.span)?)?,
            i.span,
        )),
        None => Err(SpannedError::new(
            "FFI definition should have an inner <func> tag",
            node.span,
        )),
    }?;

    // ------------------------- args -------------------------
    let args = node
        .get_many_tags_by_name("arg")
        .into_iter()
        .map(|arg| {
            if arg.self_closing {
                return Err(SpannedError::new(
                    "`arg` tag cannot be self-closing",
                    arg.span,
                ));
            }

            if arg.children_len() != 1 || !arg.all_tags() {
                return Err(SpannedError::new(
                    "`arg` tag must contain only one child",
                    arg.span,
                ));
            }

            match arg.get_tag_at(0) {
                Some(ch) => Ok(Spanned::new(parse_type_raw(ch)?, ch.span)),
                None => Err(SpannedError::new(
                    "Arg block should contain a type",
                    arg.span,
                )),
            }
        })
        .collect::<Result<Vec<Spanned<UVType>>, SpannedError>>()?;

    Ok(ASTBlockType::FFIDefinition(Box::new(FFIDefinition {
        name,
        dll,
        func,
        arguments: args,
        return_type: validate_and_parse_inner_type_block(node, "returns")?,
        span: node.span,
    })))
}
