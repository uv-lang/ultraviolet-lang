use ultraviolet_core::{
    errors::SpannedError,
    traits::{EnvironmentTrait, frontend::Positional},
    types::{
        EnvRef, Environment,
        frontend::{
            Spanned,
            ast::{ForLoop, WhileLoop},
            number::UVNumberType,
            typechecker::{TControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::Typechecker;

impl Typechecker {
    /// Typecheck while loop
    pub fn check_while_loop(
        &self,
        wl: &Spanned<Box<WhileLoop>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = self.typecheck(&wl.test, env.clone())?;
        let test = cf.ty.clone();

        UVType::Boolean
            .is_assignable_from_many(&test)
            .map_err(|t| {
                SpannedError::new(
                    format!(
                        "While loop allows only `bool` for test block, but `{}` provided",
                        t
                    ),
                    t.get_span(),
                )
            })?;

        let cfi = self.analyze_group(&wl.body, Environment::new_child(env.clone()))?;

        cf.set_ty(UVType::Void, wl.get_span());
        cf.extend_returns(cfi.returns);
        Ok(cf)
    }

    /// Typecheck for loop
    pub fn check_for_loop(
        &self,
        fl: &Spanned<ForLoop>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let child_env = Environment::new_child(env.clone());
        let mut cf = TControlFlow::new_void(fl.get_span());

        let cf_start = self.typecheck(&fl.start, env.clone())?;
        cf.extend_returns(cf_start.returns);

        let start_ty = UVType::check_all_types(&cf_start.ty)
            .map_err(|t| SpannedError::new(format!("Type mismatch: {}", t), t.get_span()))?;

        let start = match start_ty {
            UVType::Number(n) => n,
            _ => {
                return Err(SpannedError::new(
                    "Type mismatch for `for` start. Expected number",
                    fl.start.get_span(),
                ));
            },
        };

        let cf_end = self.typecheck(&fl.end, env.clone())?;
        cf.extend_returns(cf_end.returns);

        let end_ty = UVType::check_all_types(&cf_end.ty)
            .map_err(|t| SpannedError::new(format!("Type mismatch: {}", t), t.get_span()))?;

        let end = match end_ty {
            UVType::Number(n) => n,
            _ => {
                return Err(SpannedError::new(
                    "Type mismatch for `for` end. Expected number",
                    fl.end.get_span(),
                ));
            },
        };

        let step = if let Some(step) = &fl.step {
            let cf_step = self.typecheck(step, env.clone())?;
            cf.extend_returns(cf_step.returns);

            let step_ty = UVType::check_all_types(&cf_step.ty)
                .map_err(|t| SpannedError::new(format!("Type mismatch: {}", t), t.get_span()))?;
            match step_ty {
                UVType::Number(n) => n,
                _ => {
                    return Err(SpannedError::new(
                        "Type mismatch for `for` step. Expected number",
                        step.get_span(),
                    ));
                },
            }
        } else {
            start.clone()
        };

        if !UVNumberType::all_eq(&[&start, &end, &step]) {
            return Err(SpannedError::new(
                "All loop parameters should be same type",
                fl.get_span(),
            ));
        }

        child_env.borrow_mut().define_variable(
            &fl.iterator.value,
            UVTypeVariable::new_from(UVType::Number(start), true),
        );

        let cf_group = self.analyze_group(&fl.body, child_env)?;
        cf.extend_returns(cf_group.returns);
        Ok(cf)
    }
}
