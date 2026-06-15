use crate::Evaluator;
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::{
            Spanned,
            ast::{VariableAccess, VariableAssign, VariableDefinition},
        },
    },
};

impl Evaluator {
    /// Define variable
    pub fn define_variable(
        &self,
        var_def: &Spanned<VariableDefinition>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        if env.borrow().find_var(var_def.name.value.clone()).is_some() {
            return Err(SpannedError::new(
                format!("Variable `{}` already defined", var_def.name.value),
                var_def.get_span(),
            ));
        }

        match self.eval_single(&var_def.value.value, env.clone())? {
            ControlFlow::Simple(value) => {
                env.borrow_mut().define_variable(
                    var_def.name.value.clone(),
                    RTVariable::new_from(value.clone(), var_def.is_const),
                );
                Ok(ControlFlow::Simple(UVRTValue::Void))
            },
            cf => Ok(cf),
        }
    }

    /// Access variable by value
    pub fn access_variable(
        &self,
        var_acc: &Spanned<VariableAccess>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        match env.borrow().find_var(var_acc.name.clone()) {
            Some(sym) => Ok(ControlFlow::Simple(sym.borrow().clone().value)),
            None => Err(SpannedError::new(
                format!("Name `{}` not defined", var_acc.name),
                var_acc.get_span(),
            )),
        }
    }

    /// Assign to a variable
    pub fn assign_variable(
        &self,
        assign_var: &Spanned<VariableAssign>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let sym = env
            .borrow()
            .find_var(assign_var.name.clone())
            .ok_or_else(|| {
                SpannedError::new(
                    format!("Variable `{}` not defined", assign_var.name),
                    assign_var.get_span(),
                )
            })?;

        if sym.borrow().constant {
            return Err(SpannedError::new(
                "Cannot assign to a constant variable",
                assign_var.get_span(),
            ));
        }

        let new_env = Environment::new_child(env);
        let result = self.eval_single(&assign_var.value.value, new_env)?;

        if let ControlFlow::Simple(uvvalue) = result {
            (*sym.borrow_mut()).value = uvvalue;
            Ok(ControlFlow::Simple(UVRTValue::Void))
        } else {
            Ok(result)
        }
    }
}
