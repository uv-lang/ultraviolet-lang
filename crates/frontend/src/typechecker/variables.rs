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

        let mut var = var_rc.borrow_mut();

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

        if let UVType::ReferenceBatch(r) = &mut var.value {
            for reference in t {
                let UVType::ReferenceBatch(rr) = &reference.value else {
                    unreachable!()
                };

                r.extend(rr.clone());
            }
        }

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

        if let UVType::ReferenceBatch(rb) = &borrowed.value {
            for r in rb.iter() {
                r.check_references_lifetime(va.get_span(), false)?;
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
        let val = UVType::ReferenceBatch(vec![ReferenceType::new_referenced(
            var_rc.borrow().value.clone(),
            Spanned::new(Rc::downgrade(&var_rc), rc.get_span()),
        )]);

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

        let all_types: Vec<Spanned<UVType>> = match &borrowed.value {
            UVType::ReferenceBatch(rb) => {
                let mut acc: Vec<Spanned<UVType>> = Vec::new();
                for r in rb.iter() {
                    let v = r.check_references_lifetime(dr.get_span(), false)?;
                    acc.append(&mut ReferenceType::get_types(&v));
                }
                acc
            },
            _ => {
                return Err(SpannedError::new(
                    "Cannot dereference primitive value",
                    dr.get_span(),
                ));
            },
        };

        Ok(TControlFlow {
            ty: all_types,
            returns: Vec::new(),
        })
    }

    /// Check dereference assign
    pub fn check_dereference_assign(
        &self,
        va: &Spanned<VariableAssign>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let reference = env.borrow().find_var(&va.name)?;

        let borrowed = reference.borrow();
        let all_refs = match &borrowed.value {
            UVType::ReferenceBatch(rb) => {
                let mut acc = Vec::new();
                for r in rb.iter() {
                    let mut v = r.check_references_lifetime(va.get_span(), true)?;
                    acc.append(&mut v);
                }
                acc
            },
            _ => {
                return Err(SpannedError::new(
                    "Cannot dereference primitive value",
                    va.get_span(),
                ));
            },
        };

        let mut cf = self.typecheck(&va.value.value, env.clone())?;
        let t = cf.ty.clone();
        let common_t = UVType::check_all_types(&t)
            .map_err(|t| SpannedError::new(format!("Type mismatch: {}", t), t.get_span()))?;

        for reference_t in ReferenceType::get_types(&all_refs) {
            reference_t.is_assignable_from_many(&t).map_err(|t| {
                SpannedError::new(
                    format!(
                        "Expected type `{}`, got `{}` for reference `{}`",
                        reference_t,
                        t,
                        va.name.join(".")
                    ),
                    t.get_span(),
                )
            })?;
        }

        for target in all_refs {
            target.borrow_mut().value = common_t.clone();
        }
        cf.set_ty(UVType::Void, va.get_span());

        Ok(cf)
    }
}
