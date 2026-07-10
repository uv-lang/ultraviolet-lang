use std::rc::Rc;

use crate::typechecker::Typechecker;
use rand::{Rng, distr::Alphanumeric};
use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, UVDisplay, ast::IsAssignable},
    },
    types::{
        EnvRef, Environment,
        frontend::{
            Span, Spanned,
            ast::{ASTBlockType, FunctionCall, FunctionDefinition},
            typechecker::{ControlFlow, UVTypeVariable},
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

enum TypecheckArgsResult {
    Types(Vec<UVType>),
    Flow(ControlFlow),
}

impl Typechecker {
    /// Typecheck function definition
    pub fn check_function_definition(
        &self,
        fd: &Spanned<FunctionDefinition>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let inner_env = Environment::new_child(env.clone());

        let mut args = Vec::new();
        let mut trailing_optional = false;
        for arg in &fd.arguments {
            // Trailing optional check
            match arg.arg_type.value {
                UVType::Optional(_) => trailing_optional = true,
                _ if trailing_optional => {
                    return Err(SpannedError::new(
                        "Non-optional argument cannot be trailing",
                        arg.get_span(),
                    ));
                },
                _ => {},
            }
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

        let body = match self.analyze_group(&fd.body, inner_env.clone())? {
            ControlFlow::Return(t) => t,
            ControlFlow::Simple(_) => UVType::Void,
        };

        let intercepted_names = inner_env
            .borrow()
            .interceptor
            .as_ref()
            .map(|i| i.intercepted_names.borrow().clone())
            .unwrap_or_default();

        fd.value.moved_symbols.replace(intercepted_names);

        if body != exp {
            return Err(SpannedError::new(
                format!(
                    "Function body should return `{}` type, but `{}` found",
                    exp, body
                ),
                fd.get_span(),
            ));
        }

        Ok(ControlFlow::Simple(returns))
    }

    /// Typecheck function call
    pub fn check_function_call(
        &self,
        fc: &Spanned<FunctionCall>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let simplified_name = fc.name.join(".");
        let Some(var) = env.borrow().find_var(&fc.name) else {
            return Err(SpannedError::new(
                format!("Function `{}` not found", simplified_name),
                fc.get_span(),
            ));
        };

        let args_types = match self.check_args(&fc.args, env.clone())? {
            TypecheckArgsResult::Types(t) => t,
            TypecheckArgsResult::Flow(cf) => return Ok(cf),
        };

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

                Ok(ControlFlow::Simple(f.returns.clone()))
            },

            UVType::Function(f) => {
                self.validate_args(&f.args, &args_types, &simplified_name, fc.get_span())?;
                Ok(ControlFlow::Simple(f.returns.clone()))
            },

            _ => Err(SpannedError::new(
                format!("`{}` is not callable", simplified_name),
                fc.get_span(),
            )),
        }
    }

    /// Get all types of args
    fn check_args(
        &self,
        args: &Vec<Spanned<ASTBlockType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TypecheckArgsResult, SpannedError> {
        let mut args_types = Vec::new();
        for arg in args {
            let t = match self.typecheck(&arg.value, env.clone())? {
                ControlFlow::Simple(t) => t,
                cf => return Ok(TypecheckArgsResult::Flow(cf)),
            };

            args_types.push(t);
        }

        Ok(TypecheckArgsResult::Types(args_types))
    }

    /// Validate function call args
    fn validate_args(
        &self,
        expected: &[UVType],
        actual: &[UVType],
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
            if !a.is_assignable_from(b) {
                return Err(SpannedError::new(
                    format!(
                        "Argument #{} for function `{}` mismatch. Expected `{}`, but `{}` provided ",
                        i + 1,
                        name,
                        a,
                        b
                    ),
                    span,
                ));
            }
        }

        Ok(())
    }
}
