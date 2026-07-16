use std::rc::Rc;

use crate::Evaluator;
use ultraviolet_core::{
    errors::SpannedError,
    traits::{EnvironmentTrait, frontend::Positional},
    types::{
        EnvRef,
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
        Ok(ControlFlow::Simple(
            env.borrow().find_var(&var_acc.name)?.borrow().clone().value,
        ))
    }

    /// Assign to a variable
    pub fn assign_variable(
        &self,
        assign_var: &Spanned<VariableAssign>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let sym = env.borrow().find_var(&assign_var.name)?;
        let result = self.eval_single(&assign_var.value.value, env)?;

        if let ControlFlow::Simple(uvvalue) = result {
            (*sym.borrow_mut()).value = uvvalue;
            Ok(ControlFlow::Simple(UVRTValue::Void))
        } else {
            Ok(result)
        }
    }

    /// Creates a reference to a variable
    pub fn create_reference(
        &self,
        reference_create: &Spanned<VariableAccess>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let var = env.borrow().find_var(&reference_create.name)?;

        Ok(ControlFlow::Simple(UVRTValue::Reference(Rc::downgrade(
            &var,
        ))))
    }

    /// Dereferences a reference
    pub fn dereference(
        &self,
        dereference: &Spanned<VariableAccess>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let var = env.borrow().find_var(&dereference.name)?;

        let UVRTValue::Reference(r) = &var.borrow().value else {
            // SAFETY: This check is performed by typechecker
            unreachable!()
        };

        let Some(upgraded) = r.upgrade() else {
            return Err(SpannedError::new(
                "Value at this pointer is freed",
                dereference.get_span(),
            ));
        };

        // FIXME: Should the value be copied here?
        Ok(ControlFlow::Simple(upgraded.borrow().value.clone()))
    }

    /// Assign to a referenced variable
    pub fn assign_reference(
        &self,
        assign_ref: &Spanned<VariableAssign>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let sym = env.borrow().find_var(&assign_ref.name)?;

        let UVRTValue::Reference(r) = &sym.borrow().value else {
            // SAFETY: This check is performed by typechecker
            unreachable!()
        };

        let Some(strong) = r.upgrade() else {
            // SAFETY: This check is performed by typechecker
            unreachable!()
        };

        let result = self.eval_single(&assign_ref.value.value, env)?;

        if let ControlFlow::Simple(uvvalue) = result {
            (*strong.borrow_mut()).value = uvvalue;
            Ok(ControlFlow::Simple(UVRTValue::Void))
        } else {
            Ok(result)
        }
    }
}
