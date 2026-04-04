use crate::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::frontend::{Span, Spanned},
};

#[derive(Debug, Clone, PartialEq)]
pub struct UVParseNode {
    pub name: String,
    pub children: Vec<UVParseBody>,

    /// Node is self-closing `<name />`
    pub self_closing: bool,

    /// Node extra param `<name extra_param>...</ node>`
    pub extra_param: String,

    pub span: Span,
}

impl UVParseNode {
    /// Get count of children
    pub fn children_len(&self) -> usize {
        self.children.len()
    }

    /// Get inner TAG child by name
    pub fn get_one_tag_by_name(&self, name: &str) -> Option<&UVParseNode> {
        self.children.iter().find_map(|ch| match ch {
            UVParseBody::Tag(node) if node.name == name => Some(node.as_ref()),
            _ => None,
        })
    }

    /// Get inner TAG children by name
    pub fn get_many_tags_by_name(&self, name: &str) -> Vec<&UVParseNode> {
        self.children
            .iter()
            .filter_map(|ch| match ch {
                UVParseBody::Tag(node) if node.name == name => Some(node.as_ref()),
                _ => None,
            })
            .collect()
    }

    /// Get first inner literal
    pub fn get_inner_literal(&self) -> Option<&Spanned<String>> {
        self.children.iter().find_map(|ch| match ch {
            UVParseBody::String(literal) => Some(literal),
            _ => None,
        })
    }

    /// Get inner child at provided index
    pub fn get_child_at(&self, pos: usize) -> Option<&UVParseBody> {
        match self.children.get(pos) {
            Some(child) => Some(child),
            None => None,
        }
    }

    /// Get inner TAG at provided index
    pub fn get_tag_at(&self, pos: usize) -> Option<&UVParseNode> {
        match self.children.get(pos) {
            Some(UVParseBody::Tag(child)) => Some(child),
            _ => None,
        }
    }

    /// Check if all children is literals
    pub fn all_literals(&self) -> bool {
        self.children
            .iter()
            .all(|ch| matches!(ch, UVParseBody::String(_)))
    }

    /// Check if all children is tags
    pub fn all_tags(&self) -> bool {
        self.children
            .iter()
            .all(|ch| matches!(ch, UVParseBody::Tag(_)))
    }

    /// Search extra children, that not included in white list
    pub fn search_extra_children(&self, white_list: Vec<impl Into<String>>) -> Vec<UVParseBody> {
        let white_list_strings: Vec<String> = white_list.into_iter().map(|s| s.into()).collect();
        self.children
            .iter()
            .filter(|ch| match ch {
                UVParseBody::Tag(node) => !white_list_strings.contains(&node.name),
                _ => true,
            })
            .cloned()
            .collect()
    }

    /// Get all nested tags (nodes)
    pub fn get_all_tags(&self) -> Vec<&UVParseNode> {
        self.children
            .iter()
            .filter_map(|ch| match ch {
                UVParseBody::Tag(node) => Some(node.as_ref()),
                UVParseBody::String(_) => None,
            })
            .collect()
    }
}

impl Positional for UVParseNode {
    fn get_span(&self) -> Span {
        self.span
    }
}

// -------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum UVParseBody {
    String(Spanned<String>),
    Tag(Box<UVParseNode>),
}

impl Positional for UVParseBody {
    fn get_span(&self) -> Span {
        match self {
            UVParseBody::String(type_with_span) => type_with_span.span,
            UVParseBody::Tag(parse_node) => parse_node.span,
        }
    }
}

impl<T: Positional + Clone> UnwrapOptionError<T> for Option<T> {
    fn unwrap_or_spanned(&self, parent_span: Span) -> Result<T, crate::errors::SpannedError> {
        self.clone().ok_or(SpannedError::new(
            "[INTERNAL ERROR] Cannot unwrap Option value",
            parent_span,
        ))
    }
}

#[derive(Debug)]
pub enum UVParseState {
    Unknown,
    TagName,
    TagBody,
    ExtraParam,
    ClosingAngleBracketOpeningTag,
    ClosingAngleBracketClosingTag,
    ClosingTagName,
}
