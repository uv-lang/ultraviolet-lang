use crate::{
    builtins::{
        constants::{get_builtin_constant, is_builtin_constant},
        functions::{execute_builtin_function, is_builtin_function},
    },
    eval::{
        conditional_op::eval_conditional_op,
        program::eval_program,
        variables::{access_variable, assign_variable, define_variable},
    },
};
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        backend::{ControlFlow, EnvRef, Environment},
        frontend::ast::{ASTBlockType, UVValue},
    },
};
mod conditional_op;
mod program;
mod variables;

pub fn eval(node: &ASTBlockType, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    Ok(match node {
        // Main program and others service blocks
        ASTBlockType::Program(program_block) => eval_program(program_block, env)?,
        ASTBlockType::HeadBlock(blocks) | ASTBlockType::MainBlock(blocks) => eval_block(blocks, env)?,

        // Variables things
        ASTBlockType::VariableDefinition(def) => define_variable(def, env)?,
        ASTBlockType::VariableAssignment(var_assign) => assign_variable(var_assign, env)?,

        // Builtin constants
        // TODO: Local variables should override builtin constants
        ASTBlockType::VariableAccess(var_acc) if is_builtin_constant(&var_acc.name) => {
            get_builtin_constant(&var_acc.name)
        },

        ASTBlockType::VariableAccess(var_acc) => access_variable(var_acc, env)?,

        // Functions things
        ASTBlockType::FunctionDefinition(_function_definition) => todo!(),
        ASTBlockType::FunctionCall(fc) if is_builtin_function(&fc.name) => execute_builtin_function(fc, env)?,
        ASTBlockType::FunctionCall(_function_call) => todo!(),

        ASTBlockType::ConditionalOp(co) => eval_conditional_op(co, env)?,
        ASTBlockType::MathOp(_math_op) => todo!(),
        ASTBlockType::LogicalOp(_logical_op) => todo!(),
        ASTBlockType::CompareOp(_compare_op) => todo!(),
        ASTBlockType::ForLoop(_for_loop) => todo!(),
        ASTBlockType::WhileLoop(_while_loop) => todo!(),
        ASTBlockType::Value(val) => ControlFlow::Simple(val.value.clone()),
        ASTBlockType::GroupBlock(block) => eval_block(block, env)?,
        ASTBlockType::Return(block) => eval_return(block, env)?,
    })
}

/// Eval every block in node vector
fn eval_block(nodes: &Vec<ASTBlockType>, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    let new_env = Environment::new_child(env);

    let mut last_eval_simple_val = UVValue::Void;
    for node in nodes {
        match eval(node, new_env.clone())? {
            ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),

            // FIXME: Должен ли блок возвращать последнее вычисленное значение?
            ControlFlow::Simple(val) => last_eval_simple_val = val,
        }
    }

    Ok(ControlFlow::Simple(last_eval_simple_val))
}

/// Evaluate return block
fn eval_return(node: &ASTBlockType, env: EnvRef) -> Result<ControlFlow, SpannedError> {
    match eval(node, env)? {
        ControlFlow::Simple(val) | ControlFlow::Return(val) => Ok(ControlFlow::Return(val)),
    }
}
