use std::{ops::Deref, slice};

use ultraviolet_core::{
    errors::SpannedError,
    traits::{EnvironmentTrait, frontend::Positional},
    types::{
        EnvRef, Environment,
        frontend::{
            Spanned,
            ast::Namespace,
            typechecker::{TControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::Typechecker;

impl Typechecker {
    /// typecheck module block
    pub fn typecheck_namespace(
        &self,
        ns: &Spanned<Namespace>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        if env.borrow().find_var(slice::from_ref(&ns.name)).is_ok() {
            return Err(SpannedError::new(
                format!(
                    "Cannot create namespace with name {}: Name already exists",
                    ns.name
                ),
                ns.get_span(),
            ));
        }

        let namespace = Environment::new_child_weak(env.clone());
        let mut cf = TControlFlow::new_void(ns.get_span());
        for node in &ns.body {
            let cfi = self.typecheck(node, namespace.clone())?;
            cf.extend_from(cfi);
        }

        env.borrow_mut().define_variable(
            ns.name.deref().clone(),
            UVTypeVariable::new_from(UVType::Namespace(namespace), true),
        );

        cf.set_ty(UVType::Void, ns.get_span());
        Ok(cf)
    }
}
