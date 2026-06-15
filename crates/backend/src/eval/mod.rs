use crate::{Evaluator, eval::ops::EvalOps};
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
mod ffi;
mod functions;
mod logical;
mod loops;
mod math;
mod modules;
mod ops;
mod variables;

impl Evaluator {
    pub fn eval_single(
        &self,
        node: &ASTBlockType,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        Ok(match node {
            // Main program and others service blocks
            ASTBlockType::CodeBlock(code) | ASTBlockType::ModuleBlock(code) => {
                self.eval_block(code, env)?
            },

            // Variables things
            ASTBlockType::VariableDefinition(def) => self.define_variable(def, env)?,
            ASTBlockType::VariableAssignment(var_assign) => {
                self.assign_variable(var_assign, env)?
            },
            ASTBlockType::VariableAccess(var_acc) => self.access_variable(var_acc, env)?,

            // Functions things
            ASTBlockType::FunctionDefinition(function_definition) => {
                self.define_function(function_definition, env)?
            },
            ASTBlockType::FunctionCall(function_call) => self.call_function(function_call, env)?,

            ASTBlockType::ConditionalOp(co) => self.eval_conditional_op(co, env)?,
            ASTBlockType::MathOp(math_op) => math_op.eval(env, self)?,
            ASTBlockType::LogicalOp(logical_op) => logical_op.eval(env, self)?,

            ASTBlockType::CompareOp(compare_op) => compare_op.eval(env, self)?,
            ASTBlockType::ForLoop(for_loop) => self.eval_for_loop(for_loop, env)?,
            ASTBlockType::WhileLoop(while_loop) => self.eval_while_loop(while_loop, env)?,

            ASTBlockType::Value(val) => {
                ControlFlow::Simple(UVRTValue::from_uvvalue(val.value.clone()))
            },
            ASTBlockType::GroupBlock(block) => self.eval_block(block, env)?,
            ASTBlockType::Return(block) => self.eval_return(block, env)?,

            ASTBlockType::Break(_) => ControlFlow::Break,
            ASTBlockType::Continue(_) => ControlFlow::Continue,

            ASTBlockType::FFIDefinition(ffi_def) => self.load_dll(ffi_def, env)?,

            ASTBlockType::ModuleImport(mi) => self.eval_module(mi, env)?,
            ASTBlockType::ModuleExport(me) => self.eval_export(me, env)?,
        })
    }

    /// Eval every block in node vector
    fn eval_block(
        &self,
        nodes: &Vec<Spanned<ASTBlockType>>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let new_env = Environment::new_child(env);

        let mut last_eval_simple_val = UVRTValue::Void;
        for node in nodes {
            match self.eval_single(&node.value, new_env.clone())? {
                ControlFlow::Simple(val) => last_eval_simple_val = val,
                cf => return Ok(cf),
            }
        }

        Ok(ControlFlow::Simple(last_eval_simple_val))
    }

    /// Evaluate return block
    fn eval_return(
        &self,
        body: &Spanned<Option<Box<ASTBlockType>>>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(ref b) = body.value else {
            return Ok(ControlFlow::Return(UVRTValue::Void));
        };

        match self.eval_single(b, env)? {
            ControlFlow::Simple(val) | ControlFlow::Return(val) => Ok(ControlFlow::Return(val)),
            ControlFlow::Break | ControlFlow::Continue => Ok(ControlFlow::Simple(UVRTValue::Void)),
        }
    }
}
