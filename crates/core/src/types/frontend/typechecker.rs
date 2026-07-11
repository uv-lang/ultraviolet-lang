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
}

impl UVTypeVariable {
    /// Create new variable from value
    pub fn new_from(t: UVType, constant: bool) -> Self {
        Self { value: t, constant }
    }
}

impl GetVariableContainedEnvironment for UVTypeVariable {
    type Out = UVTypeVariable;
    fn get_variable_contained_env(&self) -> Option<EnvRef<Self::Out>> {
        match &self.value {
            UVType::Module(env) => Some(env.clone()),
            _ => None,
        }
    }
}
