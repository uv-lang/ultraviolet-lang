use crate::{
    EvalOps,
    eval::{
        conditional_op::eval_conditional_op,
        functions::{call_function, define_function},
        loops::{eval_for_loop, eval_while_loop},
        program::eval_program,
        variables::{access_variable, assign_variable, define_variable},
    },
};
use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef, Environment,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::{Spanned, ast::ASTBlockType},
    },
};
mod compare;
mod conditional_op;
mod functions;
mod logical;
mod loops;
mod math;
mod program;
mod variables;

pub fn eval(node: &ASTBlockType, env: EnvRef<RTVariable>) -> Result<ControlFlow, SpannedError> {
    Ok(match node {
        // Main program and others service blocks
        ASTBlockType::Program(program_block) => eval_program(program_block, env)?,
        ASTBlockType::HeadBlock(blocks) | ASTBlockType::MainBlock(blocks) => {
            eval_block(blocks, env)?
        },

        // Variables things
        ASTBlockType::VariableDefinition(def) => define_variable(def, env)?,
        ASTBlockType::VariableAssignment(var_assign) => assign_variable(var_assign, env)?,
        ASTBlockType::VariableAccess(var_acc) => access_variable(var_acc, env)?,

        // Functions things
        ASTBlockType::FunctionDefinition(function_definition) => {
            define_function(function_definition, env)?
        },
        ASTBlockType::FunctionCall(function_call) => call_function(function_call, env)?,

        ASTBlockType::ConditionalOp(co) => eval_conditional_op(co, env)?,
        ASTBlockType::MathOp(math_op) => math_op.eval(env)?,
        ASTBlockType::LogicalOp(logical_op) => logical_op.eval(env)?,
        ASTBlockType::CompareOp(compare_op) => compare_op.eval(env)?,
        ASTBlockType::ForLoop(for_loop) => eval_for_loop(for_loop, env)?,
        ASTBlockType::WhileLoop(while_loop) => eval_while_loop(while_loop, env)?,
        ASTBlockType::Value(val) => ControlFlow::Simple(UVRTValue::from_uvvalue(val.value.clone())),
        ASTBlockType::GroupBlock(block) => eval_block(block, env)?,
        ASTBlockType::Return(block) => eval_return(block, env)?,

        ASTBlockType::Break(_) => ControlFlow::Break,
        ASTBlockType::Continue(_) => ControlFlow::Continue,
    })
}

/// Eval every block in node vector
fn eval_block(
    nodes: &Vec<ASTBlockType>,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let new_env = Environment::new_child(env);

    let mut last_eval_simple_val = UVRTValue::Void;
    for node in nodes {
        match eval(node, new_env.clone())? {
            // FIXME: Should the block return the last calculated value?
            ControlFlow::Simple(val) => last_eval_simple_val = val,
            cf => return Ok(cf),
        }
    }

    Ok(ControlFlow::Simple(last_eval_simple_val))
}

/// Evaluate return block
fn eval_return(
    body: &Spanned<Option<Box<ASTBlockType>>>,
    env: EnvRef<RTVariable>,
) -> Result<ControlFlow, SpannedError> {
    let Some(ref b) = body.value else {
        return Ok(ControlFlow::Return(UVRTValue::Void));
    };

    match eval(b, env)? {
        ControlFlow::Simple(val) | ControlFlow::Return(val) => Ok(ControlFlow::Return(val)),
        ControlFlow::Break | ControlFlow::Continue => Ok(ControlFlow::Simple(UVRTValue::Void)),
    }
}
