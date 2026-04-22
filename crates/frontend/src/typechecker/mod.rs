use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::GetType,
    types::{
        EnvRef, Environment,
        frontend::{
            ast::{ASTBlockType, ProgramBlock, UVType},
            typechecker::{ControlFlow, UVTypeVariable},
        },
    },
};

use crate::typechecker::{
    functions::{check_function_call, check_function_definition},
    operators::{check_compare_op, check_conditional_op, check_logical_op, check_math_op},
    variables::{check_variable_access, check_variable_assign, check_variable_definition},
};

mod functions;
mod operators;
mod variables;

pub fn typecheck(
    block: &ASTBlockType,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    Ok(match block {
        ASTBlockType::Program(p) => analyze_program(p, env)?,
        ASTBlockType::MainBlock(m) => analyze_group(&m.value, env)?,

        ASTBlockType::VariableDefinition(vd) => check_variable_definition(vd, env)?,
        ASTBlockType::VariableAssignment(va) => check_variable_assign(va, env)?,
        ASTBlockType::VariableAccess(va) => check_variable_access(va, env)?,

        ASTBlockType::FunctionDefinition(fd) => check_function_definition(fd, env)?,
        ASTBlockType::FunctionCall(fc) => check_function_call(fc, env)?,

        ASTBlockType::ConditionalOp(co) => check_conditional_op(co, env)?,
        ASTBlockType::MathOp(mo) => check_math_op(mo, env)?,
        ASTBlockType::LogicalOp(lo) => check_logical_op(lo, env)?,
        ASTBlockType::CompareOp(co) => check_compare_op(co, env)?,

        ASTBlockType::ForLoop(_for_loop) => todo!(),
        ASTBlockType::WhileLoop(_while_loop) => todo!(),

        ASTBlockType::Value(v) => ControlFlow::Simple(v.value.get_type()),
        ASTBlockType::GroupBlock(g) => analyze_group(&g.value, env)?,
        ASTBlockType::Return(r) => analyze_return(&r.value, env)?,
        ASTBlockType::Continue(_) | ASTBlockType::Break(_) => ControlFlow::Simple(UVType::Void),

        _ => ControlFlow::Simple(UVType::Void),
    })
}

/// Analyze main program block
fn analyze_program(
    pr: &ProgramBlock,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let new_env = Environment::new_child(env);
    typecheck(&pr.main, new_env)?;

    Ok(ControlFlow::Simple(UVType::Void))
}

/// Analyze group of block
///
/// Handle return and passes upstream
/// Returns latest block type as group type
fn analyze_group(
    blocks: &Vec<ASTBlockType>,
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
