use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{
        Positional,
        ast::{GetBlockName, StringToUVType},
        token_parser::UnwrapOptionError,
    },
    types::frontend::{
        Spanned,
        tokens::UVParseNode,
        types::{UVFunctionType, UVType},
    },
};

/// Parse Ultraviolet type into UVType
pub fn parse_type_raw(node: &UVParseNode) -> Result<UVType, SpannedError> {
    // Allow clippy not to collapse match-if, since this is exactly the behavior we need
    #[allow(clippy::collapsible_match)]
    match node.name.as_str() {
        "union" | "fn" | "optional" => {
            if node.self_closing {
                return Err(SpannedError::new(
                    "This type cannot be individual",
                    node.get_span(),
                ));
            }
        },
        _ if !node.self_closing => {
            return Err(SpannedError::new(
                "All type tags must be self-closing",
                node.get_span(),
            ));
        },
        _ => {},
    }

    if let Some(t) = node.name.as_str().to_uvtype() {
        return Ok(t);
    }

    Ok(match node.name.as_str() {
        "union" => parse_union(node)?,
        "fn" => parse_fn_type(node)?,

        // TODO: Make this accessible from user env
        // "optional" => parse_optional(node)?,
        _ => {
            return Err(SpannedError::new(
                format!("Unknown type `{}`", node.name),
                node.get_span(),
            ));
        },
    })
}

fn parse_union(node: &UVParseNode) -> Result<UVType, SpannedError> {
    if !node.all_tags() {
        return Err(SpannedError::new(
            "All children inside union tag must be known types",
            node.get_span(),
        ));
    }

    if node.children_len() == 0 {
        return Err(SpannedError::new(
            "Union type cannot be empty",
            node.get_span(),
        ));
    }

    if node.children_len() == 1 {
        let t = node.get_tag_at(0).unwrap_or_spanned(node.get_span())?;
        return parse_type_raw(t);
    }

    let types = node
        .get_all_tags()
        .into_iter()
        .map(parse_type_raw)
        .collect::<Result<Vec<UVType>, SpannedError>>()?;

    Ok(UVType::new_union(types))
}

/// Try to find inner type tag and parse its children types
pub fn validate_and_parse_inner_type_block(
    node: &UVParseNode,
    tag_name: impl Into<String>,
) -> Result<Option<Spanned<UVType>>, SpannedError> {
    let name = tag_name.into();

    match node.get_one_tag_by_name(name.as_str()) {
        Some(c) if c.self_closing => Err(SpannedError::new(
            format!("`{name}` tag cannot be self-closing"),
            c.get_span(),
        )),
        Some(ch) if ch.children_len() != 1 || !ch.all_tags() => Err(SpannedError::new(
            format!("`{name}` tag must contain only one child"),
            ch.get_span(),
        )),
        Some(ch) => Ok(Some(Spanned::new(
            parse_type_raw(ch.get_tag_at(0).unwrap_or_spanned(ch.get_span())?)?,
            ch.get_span(),
        ))),
        None => Ok(None),
    }
}

/// Parse function type definition
pub fn parse_fn_type(node: &UVParseNode) -> Result<UVType, SpannedError> {
    let extra = node.search_extra_children(vec!["arg", "returns"]);

    if !extra.is_empty() {
        let first_extra = extra.first().unwrap_or_spanned(node.get_span())?;

        return Err(SpannedError::new(
            format!(
                "Found extra children {} inside function type definition",
                first_extra.get_block_name()
            ),
            first_extra.get_span(),
        ));
    }

    let mut args = Vec::new();
    let args_raw = node.get_many_tags_by_name("arg");
    for arg in args_raw {
        if !arg.all_tags() || arg.children_len() != 1 {
            return Err(SpannedError::new(
                "Function type argument should have only one nested tag",
                arg.get_span(),
            ));
        }
        args.push(parse_type_raw(
            arg.get_tag_at(0).unwrap_or_spanned(arg.get_span())?,
        )?);
    }

    let returns = node
        .get_one_tag_by_name("returns")
        .map_or(Ok(UVType::Void), parse_type_raw)?;

    Ok(UVType::Function(Box::new(UVFunctionType { args, returns })))
}

// Parse optional value
/*
fn parse_optional(node: &UVParseNode) -> Result<UVType, SpannedError> {
    let child = node.get_tag_at(0).ok_or(SpannedError::new(
        "Optional value should contain other type",
        node.get_span(),
    ))?;

    Ok(UVType::Optional(Box::new(
        parse_type_raw(child)?.flat_optional(),
    )))
}
*/
