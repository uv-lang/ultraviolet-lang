use std::rc::Rc;

use crate::typechecker::Typechecker;
use rand::{Rng, distr::Alphanumeric};
use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, UVDisplay},
    },
    types::{
        EnvRef, Environment,
        frontend::{
            Span, Spanned,
            ast::{FunctionCall, FunctionDefinition},
            typechecker::{TControlFlow, UVTypeVariable},
            types::{ReferenceType, UVBuiltinFunctionArguments, UVFunctionType, UVType},
        },
    },
};

/// Generates random str
fn random_name(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

impl Typechecker {
    /// Typecheck function definition
    pub fn check_function_definition(
        &self,
        fd: &Spanned<FunctionDefinition>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let inner_env = Environment::new_child(env.clone());

        let mut args = Vec::new();
        for arg in &fd.arguments {
            let mut arg_t = arg.arg_type.value.clone();
            if let UVType::Reference(rr) = &arg_t {
                let v = inner_env.borrow_mut().define_variable(
                    random_name(5),
                    UVTypeVariable::new_from(rr.t.clone(), false),
                );

                arg_t = UVType::Reference(Box::new(ReferenceType::new_referenced(
                    rr.t.clone(),
                    Rc::downgrade(&v),
                )));
            }

            inner_env.borrow_mut().define_variable(
                &arg.name.value,
                UVTypeVariable::new_from(arg_t.clone(), false),
            );
            args.push(arg_t);
        }

        let exp = fd
            .return_type
            .clone()
            .map(|t| t.value)
            .unwrap_or(UVType::Void);

        let mut returns = UVType::Void;
        let f = UVType::Function(Box::new(UVFunctionType {
            args,
            returns: exp.clone(),
        }));

        if let Some(name) = &fd.name {
            env.borrow_mut()
                .define_variable(&name.value, UVTypeVariable::new_from(f, true));
        } else {
            returns = f;
        }

        // enable symbols interception
        inner_env.borrow_mut().enable_interception();

        let mut body_cf = self.analyze_group(&fd.body, inner_env.clone())?;
        body_cf.returns.extend(body_cf.ty);

        let intercepted_names = inner_env
            .borrow()
            .interceptor
            .as_ref()
            .map(|i| i.intercepted_names.borrow().clone())
            .unwrap_or_default();

        fd.value.moved_symbols.replace(intercepted_names);

        exp.is_assignable_from_many(&body_cf.returns).map_err(|t| {
            SpannedError::new(
                format!(
                    "Function body should return `{}` type, but `{}` found",
                    exp, t
                ),
                t.get_span(),
            )
        })?;

        // # SAFETY:
        // Guaranteed unwrap safety, due body_cf.returns.extend above
        Ok(TControlFlow::new_ty(
            returns,
            body_cf.returns.last().unwrap().get_span(),
        ))
    }

    /// Typecheck function call
    pub fn check_function_call(
        &self,
        fc: &Spanned<FunctionCall>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let simplified_name = fc.name.join(".");
        let var = env.borrow().find_var(&fc.name)?;

        let mut cf = TControlFlow::new_void(fc.get_span());
        let mut args_types = Vec::new();
        for arg in &fc.args {
            let cfi = self.typecheck(&arg.value, env.clone())?;
            cf.extend_returns(cfi.returns);
            args_types.push(cfi.ty);
        }

        let value = &var.borrow().value;

        match value {
            UVType::BuiltInFunction(f) => {
                match &f.args {
                    UVBuiltinFunctionArguments::Args(expected) => {
                        self.validate_args(expected, &args_types, &simplified_name, fc.get_span())?
                    },
                    UVBuiltinFunctionArguments::AllOf(all_t) => self.validate_args(
                        &vec![all_t.clone(); args_types.len()],
                        &args_types,
                        &simplified_name,
                        fc.get_span(),
                    )?,
                    _ => {},
                }

                cf.add_ty(f.returns.clone(), fc.get_span());
                Ok(cf)
            },

            UVType::Function(f) => {
                self.validate_args(&f.args, &args_types, &simplified_name, fc.get_span())?;
                cf.add_ty(f.returns.clone(), fc.get_span());
                Ok(cf)
            },

            _ => Err(SpannedError::new(
                format!("`{}` is not callable", simplified_name),
                fc.get_span(),
            )),
        }
    }

    /// Validate function call args
    fn validate_args(
        &self,
        expected: &[UVType],
        actual: &[Vec<Spanned<UVType>>],
        name: &str,
        span: Span,
    ) -> Result<(), SpannedError> {
        let min_args = expected
            .iter()
            .filter(|f| !matches!(f, UVType::Optional(_)))
            .count();

        if min_args > actual.len() || expected.len() < actual.len() {
            return Err(SpannedError::new(
                format!(
                    "Function `{}` expects {} arguments, but {} provided",
                    name,
                    expected.len(),
                    actual.len()
                ),
                span,
            ));
        }

        for (i, (a, b)) in expected.iter().zip(actual).enumerate() {
            a.is_assignable_from_many(b).map_err(|t| SpannedError::new(
                    format!(
                        "Argument #{} for function `{}` mismatch. Expected `{}`, but `{}` provided ",
                        i + 1,
                        name,
                        a,
                        t
                    ),
                    t.get_span(),
                ))?;
        }

        Ok(())
    }
}
