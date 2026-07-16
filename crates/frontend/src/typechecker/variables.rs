use std::{rc::Rc, slice};

use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, UVDisplay},
    },
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::{VariableAccess, VariableAssign, VariableDefinition},
            typechecker::{TControlFlow, UVTypeVariable},
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
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = self.typecheck(&vd.value.value, env.clone())?;
        let val = cf.ty.clone();

        if let Some(expected) = &vd.expected_type {
            expected.value.is_assignable_from_many(&val).map_err(|t| {
                SpannedError::new(
                    format!(
                        "Expected type `{}`, got `{}` for variable `{}`",
                        expected.value, t, vd.name.value
                    ),
                    t.get_span(),
                )
            })?
        }

        if env.borrow().exists_in_current(slice::from_ref(&vd.name)) {
            return Err(SpannedError::new(
                format!("Variable with name {} already defined", vd.name.value),
                vd.get_span(),
            ));
        }

        let t = UVType::check_all_types(&val)
            .map_err(|t| SpannedError::new(format!("Type mismatch: {}", t), t.get_span()))?;

        env.borrow_mut()
            .define_variable(&vd.name.value, UVTypeVariable::new_from(t, vd.is_const));

        cf.set_ty(UVType::Void, vd.get_span());
        Ok(cf)
    }

    /// Check variable assignment
    pub fn check_variable_assign(
        &self,
        va: &Spanned<VariableAssign>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let var_rc = env.borrow().find_var(&va.name)?;

        if var_rc.borrow().constant {
            return Err(SpannedError::new(
                format!(
                    "Cannot assign to a constant `{}` variable",
                    va.name.join(".")
                ),
                va.get_span(),
            ));
        }

        let mut cf = self.typecheck(&va.value.value, env.clone())?;
        let t = cf.ty.clone();

        let var = var_rc.borrow_mut();

        var.value.is_assignable_from_many(&t).map_err(|t| {
            SpannedError::new(
                format!(
                    "Expected type `{}`, got `{}` for variable `{}`",
                    var.value,
                    t,
                    va.name.join(".")
                ),
                t.get_span(),
            )
        })?;

        // TODO: Write comments for this line:
        // FIXME:! This assign used for references checking!!!!
        //var.value = t.value;

        cf.set_ty(UVType::Void, va.get_span());
        Ok(cf)
    }

    /// Check variable is defined and get its type
    pub fn check_variable_access(
        &self,
        va: &Spanned<VariableAccess>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let var_rc = env.borrow().find_var(&va.name)?;
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

        Ok(TControlFlow::new_ty(borrowed.value.clone(), va.get_span()))
    }

    /// Check and validate reference creation
    pub fn check_reference_create(
        &self,
        rc: &Spanned<VariableAccess>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let var_rc = env.borrow().find_var(&rc.name)?;
        let val = UVType::Reference(Box::new(ReferenceType::new_referenced(
            var_rc.borrow().value.clone(),
            Rc::downgrade(&var_rc),
        )));

        Ok(TControlFlow::new_ty(val, rc.get_span()))
    }

    /// Check and validate dereference
    pub fn check_dereference(
        &self,
        dr: &Spanned<VariableAccess>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let var_rc = env.borrow().find_var(&dr.name)?;

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

        Ok(TControlFlow::new_ty(
            upgraded.borrow().value.clone(),
            dr.get_span(),
        ))
    }

    /// Check dereference assign
    pub fn check_dereference_assign(
        &self,
        va: &Spanned<VariableAssign>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let reference = env.borrow().find_var(&va.name)?;

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

        if strong_ref.borrow().constant {
            return Err(SpannedError::new(
                "Attempt to assign to dereferenced constant value",
                va.get_span(),
            ));
        }

        let mut cf = self.typecheck(&va.value.value, env.clone())?;
        let t = cf.ty.clone();

        let borrowed_ref = strong_ref.borrow_mut();

        borrowed_ref
            .value
            .is_assignable_from_many(&t)
            .map_err(|t| {
                SpannedError::new(
                    format!(
                        "Expected type `{}`, got `{}` for reference `{}`",
                        borrowed_ref.value,
                        t,
                        va.name.join(".")
                    ),
                    t.get_span(),
                )
            })?;

        // TODO: Write comments for this line:
        // FIXME:! This assign used for references checking!!!!
        //borrowed_ref.value = t.value;

        cf.set_ty(UVType::Void, va.get_span());
        Ok(cf)
    }
}
