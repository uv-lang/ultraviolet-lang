use crate::{
    traits::GetVariableContainedEnvironment,
    types::{EnvRef, frontend::types::UVType},
};

pub enum ControlFlow {
    Return(UVType),
    Simple(UVType),
}

/// Typecheck variable struct
#[derive(Clone)]
pub struct UVTypeVariable {
    pub value: UVType,
    pub constant: bool,

    /// A variable can also contain not a type, but another environment
    /// In this case, the variable becomes simply a container for the nested environment
    /// For example: When modules are imported, they create after themselves a wrapper that contains all their exported symbols
    pub environmental: Option<EnvRef<UVTypeVariable>>,
}

impl UVTypeVariable {
    /// Create new variable from value
    pub fn new_from(t: UVType, constant: bool) -> Self {
        Self {
            value: t,
            constant,
            environmental: None,
        }
    }

    /// Create new environment variable
    pub fn new_environmental(env: EnvRef<UVTypeVariable>) -> Self {
        Self {
            value: UVType::Unassignable,
            constant: true,
            environmental: Some(env),
        }
    }
}

impl GetVariableContainedEnvironment for UVTypeVariable {
    type Out = UVTypeVariable;
    fn get_variable_contained_env(&self) -> Option<EnvRef<Self::Out>> {
        self.environmental.clone()
    }
}
