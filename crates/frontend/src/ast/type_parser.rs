use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::GetBlockName, token_parser::UnwrapOptionError},
    types::frontend::{
        Spanned,
        tokens::UVParseNode,
        types::{UVFunctionType, UVNumberType, UVType},
    },
};

/// Parse Ultraviolet type into UVType
pub fn parse_type_raw(node: &UVParseNode) -> Result<UVType, SpannedError> {
    match node.name.as_str() {
        "union" | "fn" => {
            if node.self_closing {
                return Err(SpannedError::new(
                    "This type cannot be used as individual type",
                    node.span,
                ));
            }
        },
        _ if !node.self_closing => {
            return Err(SpannedError::new(
                "All type tags must be self-closing",
                node.span,
            ));
        },
        _ => {},
    }

    Ok(match node.name.as_str() {
        "int" => UVType::Number(UVNumberType::Int),
        "float" => UVType::Number(UVNumberType::Float),
        "str" => UVType::String,
        "bool" => UVType::Boolean,
        "null" => UVType::Null,
        "union" => parse_union(node)?,
        "fn" => parse_fn_type(node)?,
        _ => {
            return Err(SpannedError::new(
                format!("Unknown type `{}`", node.name),
                node.span,
            ));
        },
    })
}

fn parse_union(node: &UVParseNode) -> Result<UVType, SpannedError> {
    if !node.all_tags() {
        return Err(SpannedError::new(
            "All children inside union tag must be known types",
            node.span,
        ));
    }

    if node.children_len() == 0 {
        return Err(SpannedError::new("Union type cannot be empty", node.span));
    }

    if node.children_len() == 1 {
        let t = node.get_tag_at(0).unwrap_or_spanned(node.span)?;
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
            c.span,
        )),
        Some(ch) if ch.children_len() != 1 || !ch.all_tags() => Err(SpannedError::new(
            format!("`{name}` tag must contain only one child"),
            ch.span,
        )),
        Some(ch) => Ok(Some(Spanned::new(
            parse_type_raw(ch.get_tag_at(0).unwrap_or_spanned(ch.span)?)?,
            ch.span,
        ))),
        None => Ok(None),
    }
}

/// Parse function type definition
pub fn parse_fn_type(node: &UVParseNode) -> Result<UVType, SpannedError> {
    let extra = node.search_extra_children(vec!["arg", "returns"]);

    if !extra.is_empty() {
        let first_extra = extra.first().unwrap_or_spanned(node.span)?;

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
                arg.span,
            ));
        }
        args.push(parse_type_raw(
            arg.get_tag_at(0).unwrap_or_spanned(arg.span)?,
        )?);
    }

    let returns = node
        .get_one_tag_by_name("returns")
        .map_or(Ok(UVType::Void), parse_type_raw)?;

    Ok(UVType::Function(Box::new(UVFunctionType { args, returns })))
}
