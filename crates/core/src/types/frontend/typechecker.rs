use crate::types::frontend::ast::UVType;

pub enum ControlFlow {
    Return(UVType),
    Simple(UVType),
}

/// Typecheck variable struct
#[derive(Debug, Clone)]
pub struct UVTypeVariable {
    pub value: UVType,
    pub constant: bool,
}

impl UVTypeVariable {
    /// Create new variable from value
    pub fn new_from(val: UVType, constant: bool) -> Self {
        Self {
            value: val,
            constant,
        }
    }
}
