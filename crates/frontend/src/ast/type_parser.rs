use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::token_parser::UnwrapOptionError,
    types::frontend::{Spanned, ast::UVType, tokens::UVParseNode},
};

/// Parse Ultraviolet type into UVType
pub fn parse_type_raw(node: &UVParseNode) -> Result<UVType, SpannedError> {
    if node.name.eq("union") {
        if node.self_closing {
            return Err(SpannedError::new("Union cannot be used as individual type", node.span));
        }
        return parse_union(node);
    }

    if !node.self_closing {
        return Err(SpannedError::new("All type tags must be self-closing", node.span));
    }

    Ok(match node.name.as_str() {
        "int" => UVType::Int,
        "float" => UVType::Float,
        "str" => UVType::String,
        "bool" => UVType::Boolean,
        "null" => UVType::Null,
        _ => {
            return Err(SpannedError::new(format!("Unknown type `{}`", node.name), node.span));
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
            format!("`{}` tag cannot be self-closing", name),
            c.span,
        )),
        Some(ch) if ch.children_len() != 1 || !ch.all_tags() => Err(SpannedError::new(
            format!("`{}` tag must contain only one child", name),
            ch.span,
        )),
        Some(ch) => Ok(Some(Spanned::new(
            parse_type_raw(ch.get_tag_at(0).unwrap_or_spanned(ch.span)?)?,
            ch.span,
        ))),
        None => Ok(None),
    }
}
