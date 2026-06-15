use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::IsAssignable},
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::{VariableAccess, VariableAssign, VariableDefinition},
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::Typechecker;

impl Typechecker {
    /// Definition and checking of variable types
    pub fn check_variable_definition(
        &self,
        vd: &Spanned<VariableDefinition>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let mut val = match self.typecheck(&vd.value.value, env.clone())? {
            ControlFlow::Simple(uvtype) => uvtype,
            cf => return Ok(cf),
        };

        if let Some(expected) = &vd.expected_type {
            if !expected.value.is_assignable_from(&val) {
                return Err(SpannedError::new(
                    format!(
                        "Expected type `{}`, got `{}` for variable `{}`",
                        expected.value, val, vd.name.value
                    ),
                    vd.get_span(),
                ));
            }

            val = expected.value.clone();
        }

        if env.borrow().find_var(&vd.name.value).is_some() {
            return Err(SpannedError::new(
                format!("Variable with name {} already defined", vd.name.value),
                vd.get_span(),
            ));
        }

        env.borrow_mut()
            .define_variable(&vd.name.value, UVTypeVariable::new_from(val, vd.is_const));

        Ok(ControlFlow::Simple(UVType::Void))
    }

    /// Check variable assignment
    pub fn check_variable_assign(
        &self,
        va: &Spanned<VariableAssign>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(var_rc) = env.borrow().find_var(&va.name) else {
            return Err(SpannedError::new(
                format!("Variable `{}` not defined", va.name),
                va.get_span(),
            ));
        };

        let var = var_rc.borrow();
        if var.constant {
            return Err(SpannedError::new(
                format!("Cannot assign to a constant `{}` variable", va.name),
                va.get_span(),
            ));
        }

        let t = match self.typecheck(&va.value.value, env.clone())? {
            ControlFlow::Simple(uvtype) => uvtype,
            cf => return Ok(cf),
        };

        if !var.value.is_assignable_from(&t) {
            return Err(SpannedError::new(
                format!(
                    "Expected type `{}`, got `{}` for variable `{}`",
                    var.value, t, va.name
                ),
                va.get_span(),
            ));
        }

        Ok(ControlFlow::Simple(UVType::Void))
    }

    /// Check variable is defined and get its type
    pub fn check_variable_access(
        &self,
        va: &Spanned<VariableAccess>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(var_rc) = env.borrow().find_var(&va.name) else {
            return Err(SpannedError::new(
                format!("Variable `{}` not defined", va.name),
                va.get_span(),
            ));
        };

        Ok(ControlFlow::Simple(var_rc.borrow().value.clone()))
    }
}
