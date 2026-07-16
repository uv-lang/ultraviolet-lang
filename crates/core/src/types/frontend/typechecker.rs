use crate::{
    traits::{GetVariableContainedEnvironment, frontend::Positional},
    types::{
        EnvRef,
        frontend::{Span, Spanned, types::UVType},
    },
};

/// Typecheck variable struct
#[derive(Debug, Clone)]
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
            UVType::Namespace(env) => Some(env.clone()),
            _ => None,
        }
    }
}

pub struct TControlFlow {
    pub ty: Vec<Spanned<UVType>>,
    pub returns: Vec<Spanned<UVType>>,
}

impl TControlFlow {
    /// Create new void TControlFlow
    pub fn new_void(span: Span) -> Self {
        Self {
            ty: vec![Spanned::new(UVType::Void, span)],
            returns: Vec::new(),
        }
    }

    /// Create new TControlFlow from simple type
    pub fn new_ty(ty: UVType, span: Span) -> Self {
        Self {
            ty: vec![Spanned::new(ty, span)],
            returns: Vec::new(),
        }
    }

    /// Create new TControlFlow from returns type
    pub fn new_returns(returns: Spanned<UVType>) -> Self {
        Self {
            ty: vec![Spanned::new(UVType::Void, returns.get_span())],
            returns: vec![returns],
        }
    }

    /// Set current type
    pub fn set_ty(&mut self, ty: UVType, span: Span) -> &mut Self {
        self.ty = vec![Spanned::new(ty, span)];
        self
    }

    /// Add current type
    pub fn add_ty(&mut self, ty: UVType, span: Span) -> &mut Self {
        self.ty.push(Spanned::new(ty, span));
        self
    }

    /// Add return type
    pub fn add_returns(&mut self, returns: Spanned<UVType>) -> &mut Self {
        self.returns.push(returns);
        self
    }

    /// Extend return type and ty from other TControlFlow
    pub fn extend_from(&mut self, other: TControlFlow) -> &mut Self {
        self.returns.extend(other.returns);
        self.ty = other.ty;
        self
    }

    /// Extend return type from other TControlFlow
    pub fn extend_returns(&mut self, other: Vec<Spanned<UVType>>) -> &mut Self {
        self.returns.extend(other);
        self
    }
}
