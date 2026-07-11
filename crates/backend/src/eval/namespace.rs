use std::ops::Deref;

use ultraviolet_core::{
    errors::SpannedError,
    traits::EnvironmentTrait,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::{Spanned, ast::Namespace},
    },
};

use crate::Evaluator;

impl Evaluator {
    /// Eval namespace block
    pub fn eval_namespace(
        &self,
        ns: &Spanned<Namespace>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let namespace = Environment::new_child_weak(env.clone());
        let mut cf = ControlFlow::Simple(UVRTValue::Void);
        for node in &ns.body {
            match self.eval_single(node, namespace.clone())? {
                ControlFlow::Simple(_) => {},
                c => {
                    cf = c;
                    break;
                },
            }
        }

        env.borrow_mut().define_variable(
            ns.name.deref().clone(),
            RTVariable::new_from(UVRTValue::Namespace(namespace), true),
        );

        Ok(cf)
    }
}
