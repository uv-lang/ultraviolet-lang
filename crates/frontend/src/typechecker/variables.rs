use std::{rc::Rc, slice};

use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, UVDisplay, ast::IsAssignable},
    },
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::{VariableAccess, VariableAssign, VariableDefinition},
            typechecker::{ControlFlow, UVTypeVariable},
            types::{ReferenceType, UVType},
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
        let val = match self.typecheck(&vd.value.value, env.clone())? {
            ControlFlow::Simple(uvtype) => uvtype,
            cf => return Ok(cf),
        };

        if let Some(expected) = &vd.expected_type
            && !expected.value.is_assignable_from(&val)
        {
            return Err(SpannedError::new(
                format!(
                    "Expected type `{}`, got `{}` for variable `{}`",
                    expected.value, val, vd.name.value
                ),
                vd.value.value.get_span(),
            ));
        }

        if env.borrow().find_var(slice::from_ref(&vd.name)).is_some() {
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
                format!("Variable `{}` not defined", va.name.join(".")),
                va.get_span(),
            ));
        };

        let mut var = var_rc.borrow_mut();
        if var.constant {
            return Err(SpannedError::new(
                format!(
                    "Cannot assign to a constant `{}` variable",
                    va.name.join(".")
                ),
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
                    var.value,
                    t,
                    va.name.join(".")
                ),
                va.get_span(),
            ));
        }

        var.value = t;

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
                format!("Variable `{}` not defined", va.name.join(".")),
                va.get_span(),
            ));
        };

        let borrowed = var_rc.borrow();

        if let UVType::Reference(r) = &borrowed.value {
            let Some(referenced_var) = &r.reference else {
                return Err(SpannedError::new_tipped(
                    "This is a dangling reference",
                    "Report about this issue to https://github.com/Andcool-Systems/ultraviolet-lang",
                    va.get_span(),
                ));
            };

            if referenced_var.upgrade().is_none() {
                return Err(SpannedError::new(
                    "Value at this reference is freed before accessing",
                    va.get_span(),
                ));
            }
        }

        Ok(ControlFlow::Simple(borrowed.value.clone()))
    }

    /// Check and validate reference creation
    pub fn check_reference_create(
        &self,
        rc: &Spanned<VariableAccess>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(var_rc) = env.borrow().find_var(&rc.name) else {
            return Err(SpannedError::new(
                format!("Variable `{}` not defined", rc.name.join(".")),
                rc.get_span(),
            ));
        };

        Ok(ControlFlow::Simple(UVType::Reference(Box::new(
            ReferenceType::new_referenced(var_rc.borrow().value.clone(), Rc::downgrade(&var_rc)),
        ))))
    }

    /// Check and validate dereference
    pub fn check_dereference(
        &self,
        dr: &Spanned<VariableAccess>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(var_rc) = env.borrow().find_var(&dr.name) else {
            return Err(SpannedError::new(
                format!("Reference `{}` not defined", dr.name.join(".")),
                dr.get_span(),
            ));
        };

        let borrowed = var_rc.borrow();
        let UVType::Reference(rf) = &borrowed.value else {
            return Err(SpannedError::new(
                "Cannot dereference primitive value",
                dr.get_span(),
            ));
        };

        let Some(referenced_var) = &rf.reference else {
            return Err(SpannedError::new_tipped(
                "This is a dangling reference",
                "Report about this issue to https://github.com/Andcool-Systems/ultraviolet-lang",
                dr.get_span(),
            ));
        };

        let Some(upgraded) = referenced_var.upgrade() else {
            return Err(SpannedError::new(
                "Value at this reference is freed before accessing",
                dr.get_span(),
            ));
        };

        Ok(ControlFlow::Simple(upgraded.borrow().value.clone()))
    }

    /// Check dereference assign
    pub fn check_dereference_assign(
        &self,
        va: &Spanned<VariableAssign>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(reference) = env.borrow().find_var(&va.name) else {
            return Err(SpannedError::new(
                format!("Symbol {} not found", va.name.join(".")),
                va.get_span(),
            ));
        };

        let borrowed = reference.borrow();
        let UVType::Reference(r) = &borrowed.value else {
            return Err(SpannedError::new(
                "Cannot dereference non-reference variable",
                va.get_span(),
            ));
        };

        let Some(referenced_var) = &r.reference else {
            return Err(SpannedError::new_tipped(
                "This is a dangling reference",
                "Report about this issue to https://github.com/Andcool-Systems/ultraviolet-lang",
                va.get_span(),
            ));
        };

        let Some(strong_ref) = referenced_var.upgrade() else {
            return Err(SpannedError::new(
                "Value at this reference is freed before accessing",
                va.get_span(),
            ));
        };

        if strong_ref.borrow_mut().constant {
            return Err(SpannedError::new(
                "Attempt to assign to dereferenced constant value",
                va.get_span(),
            ));
        }

        let t = match self.typecheck(&va.value.value, env.clone())? {
            ControlFlow::Simple(uvtype) => uvtype,
            cf => return Ok(cf),
        };

        let mut borrowed_ref = strong_ref.borrow_mut();

        if !borrowed_ref.value.is_assignable_from(&t) {
            return Err(SpannedError::new(
                format!(
                    "Expected type `{}`, got `{}` for reference `{}`",
                    borrowed_ref.value,
                    t,
                    va.name.join(".")
                ),
                va.get_span(),
            ));
        }

        borrowed_ref.value = t;

        Ok(ControlFlow::Simple(UVType::Void))
    }
}
