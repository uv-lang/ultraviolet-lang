use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::IsAssignable},
    types::{
        EnvRef, Environment,
        frontend::{
            Span, Spanned,
            ast::{ASTBlockType, FunctionCall, FunctionDefinition},
            typechecker::{ControlFlow, UVTypeVariable},
            types::{UVBuiltinFunctionArguments, UVFunctionType, UVType},
        },
    },
};

use crate::typechecker::Typechecker;

/// Collect all symbol names, captured by function
fn collect_captured_names(nodes: &[Spanned<ASTBlockType>]) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();

    for node in nodes {
        collect_names_from_node(&node.value, &mut names);
    }

    names
}

fn collect_names_from_node(node: &ASTBlockType, names: &mut std::collections::HashSet<String>) {
    match node {
        ASTBlockType::CodeBlock(body)
        | ASTBlockType::ModuleBlock(body)
        | ASTBlockType::GroupBlock(body) => {
            names.extend(collect_captured_names(body));
        },
        ASTBlockType::VariableDefinition(def) => {
            collect_names_from_node(&def.value.value, names);
        },
        ASTBlockType::FunctionDefinition(_) => {},
        ASTBlockType::FunctionCall(call) => {
            names.insert(call.name.clone());
            for arg in &call.args {
                collect_names_from_node(&arg.value, names);
            }
        },
        ASTBlockType::VariableAssignment(assign) => {
            collect_names_from_node(&assign.value.value, names);
        },
        ASTBlockType::VariableAccess(access) => {
            names.insert(access.name.clone());
        },
        ASTBlockType::ConditionalOp(cond) => {
            collect_names_from_node(&cond.test.value, names);
            if let Some(then_body) = &cond.then_body {
                names.extend(collect_captured_names(then_body));
            }
            if let Some(else_body) = &cond.else_body {
                names.extend(collect_captured_names(else_body));
            }
        },
        ASTBlockType::MathOp(op) => {
            for operand in &op.operands {
                collect_names_from_node(&operand.value, names);
            }
        },
        ASTBlockType::LogicalOp(op) => {
            for operand in &op.operands {
                collect_names_from_node(&operand.value, names);
            }
        },
        ASTBlockType::CompareOp(op) => {
            for operand in &op.operands {
                collect_names_from_node(&operand.value, names);
            }
        },
        ASTBlockType::ForLoop(for_loop) => {
            collect_names_from_node(&for_loop.start.value, names);
            collect_names_from_node(&for_loop.end.value, names);
            if let Some(step) = &for_loop.step {
                collect_names_from_node(&step.value, names);
            }
            names.extend(collect_captured_names(&for_loop.body));
        },
        ASTBlockType::WhileLoop(while_loop) => {
            collect_names_from_node(&while_loop.test.value, names);
            names.extend(collect_captured_names(&while_loop.body));
        },
        ASTBlockType::Return(ret) => {
            if let Some(body) = &ret.value {
                collect_names_from_node(body, names);
            }
        },
        ASTBlockType::Break(_) | ASTBlockType::Continue(_) => {},
        ASTBlockType::FFIDefinition(def) => {
            collect_names_from_node(&def.dll.value, names);
            collect_names_from_node(&def.func.value, names);
        },
        ASTBlockType::ModuleImport(_) => {},
        ASTBlockType::ModuleExport(export) => {
            for access in &export.value {
                names.insert(access.name.clone());
            }
        },
        ASTBlockType::Value(_) => {},
    }
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

            inner_env.borrow_mut().define_variable(
                &arg.name.value,
                UVTypeVariable::new_from(arg.arg_type.value.clone(), true),
            );
            args.push(arg.arg_type.value.clone());
        }

        let exp = fd
            .return_type
            .clone()
            .map(|t| t.value)
            .unwrap_or(UVType::Void);

        fd.value
            .moved_symbols
            .replace(collect_captured_names(&fd.value.body).into_iter().collect());

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

        let body = match self.analyze_group(&fd.body, inner_env)? {
            ControlFlow::Return(t) => t,
            ControlFlow::Simple(_) => UVType::Void,
        };

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
        let Some(var) = env.borrow().find_var(fc.name.clone()) else {
            return Err(SpannedError::new(
                format!("Function `{}` not found", fc.name),
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
                        self.validate_args(expected, &args_types, &fc.name, fc.get_span())?
                    },
                    UVBuiltinFunctionArguments::AllOf(all_t) => self.validate_args(
                        &vec![all_t.clone(); args_types.len()],
                        &args_types,
                        &fc.name,
                        fc.get_span(),
                    )?,
                    _ => {},
                }

                Ok(ControlFlow::Simple(f.returns.clone()))
            },

            UVType::Function(f) => {
                self.validate_args(&f.args, &args_types, &fc.name, fc.get_span())?;
                Ok(ControlFlow::Simple(f.returns.clone()))
            },

            _ => Err(SpannedError::new(
                format!("`{}` is not callable", fc.name),
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
