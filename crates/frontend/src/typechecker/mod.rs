use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::GetType,
    types::{
        EnvRef, Environment,
        frontend::{
            Spanned,
            ast::ASTBlockType,
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::{
    ffi::check_ffi_definition,
    functions::{check_function_call, check_function_definition},
    loops::{check_for_loop, check_while_loop},
    operators::{check_compare_op, check_conditional_op, check_logical_op, check_math_op},
    variables::{check_variable_access, check_variable_assign, check_variable_definition},
};

mod ffi;
mod functions;
mod loops;
mod operators;
mod variables;

pub fn typecheck(
    block: &ASTBlockType,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    Ok(match block {
        ASTBlockType::CodeBlock(m) => analyze_group(&m.value, env)?,

        ASTBlockType::VariableDefinition(vd) => check_variable_definition(vd, env)?,
        ASTBlockType::VariableAssignment(va) => check_variable_assign(va, env)?,
        ASTBlockType::VariableAccess(va) => check_variable_access(va, env)?,

        ASTBlockType::FunctionDefinition(fd) => check_function_definition(fd, env)?,
        ASTBlockType::FunctionCall(fc) => check_function_call(fc, env)?,

        ASTBlockType::ConditionalOp(co) => check_conditional_op(co, env)?,
        ASTBlockType::MathOp(mo) => check_math_op(mo, env)?,
        ASTBlockType::LogicalOp(lo) => check_logical_op(lo, env)?,
        ASTBlockType::CompareOp(co) => check_compare_op(co, env)?,

        ASTBlockType::ForLoop(fl) => check_for_loop(fl, env)?,
        ASTBlockType::WhileLoop(wl) => check_while_loop(wl, env)?,

        ASTBlockType::Value(v) => ControlFlow::Simple(v.value.get_type()),
        ASTBlockType::GroupBlock(g) => analyze_group(&g.value, env)?,
        ASTBlockType::Return(r) => analyze_return(&r.value, env)?,
        ASTBlockType::Continue(_) | ASTBlockType::Break(_) => ControlFlow::Simple(UVType::Void),

        ASTBlockType::FFIDefinition(ffi_d) => check_ffi_definition(ffi_d, env)?,

        _ => ControlFlow::Simple(UVType::Void),
    })
}

/// Analyze group of block
///
/// Handle return and passes upstream
/// Returns latest block type as group type
fn analyze_group(
    blocks: &Vec<Spanned<ASTBlockType>>,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let new_env = Environment::new_child(env);

    let mut last_type = UVType::Void;
    for node in blocks {
        match typecheck(node, new_env.clone())? {
            ControlFlow::Simple(val) => last_type = val,
            cf => return Ok(cf),
        }
    }

    Ok(ControlFlow::Simple(last_type))
}

/// Analyze return block
fn analyze_return(
    body: &Option<Box<ASTBlockType>>,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(b) = body else {
        return Ok(ControlFlow::Return(UVType::Void));
    };

    match typecheck(b, env)? {
        ControlFlow::Simple(val) | ControlFlow::Return(val) => Ok(ControlFlow::Return(val)),
    }
}
