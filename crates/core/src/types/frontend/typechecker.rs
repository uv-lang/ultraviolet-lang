use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    traits::frontend::ast::GetType,
    types::frontend::ast::{ASTBlockType, UVType},
};

pub type EnvRef = Rc<RefCell<Environment>>;

/// Scope-based environment
#[derive(Default, Debug)]
pub struct Environment {
    pub symbols: HashMap<String, ()>,
    pub parent: Option<EnvRef>,
}

impl GetType for ASTBlockType {
    fn get_type(&self, env: EnvRef) -> super::ast::UVType {
        match self {
            ASTBlockType::Program(_) => UVType::Void,
            ASTBlockType::HeadBlock(_) => UVType::Void,
            ASTBlockType::MainBlock(_) => todo!(),
            ASTBlockType::VariableDefinition(_) => UVType::Void,
            ASTBlockType::FunctionDefinition(_) => UVType::Void,
            ASTBlockType::FunctionCall(_function_call) => todo!(),
            ASTBlockType::VariableAssignment(_) => UVType::Void,
            ASTBlockType::VariableAccess(_variable_access) => todo!(),
            ASTBlockType::ConditionalOp(_conditional_operator) => todo!(),
            ASTBlockType::MathOp(_math_op) => todo!(),
            ASTBlockType::LogicalOp(_logical_op) => todo!(),
            ASTBlockType::CompareOp(_compare_op) => todo!(),
            ASTBlockType::ForLoop(_) => UVType::Void,
            ASTBlockType::WhileLoop(_) => UVType::Void,
            ASTBlockType::Value(v) => v.get_type(env),
            ASTBlockType::GroupBlock(_spanned) => todo!(),
            ASTBlockType::Return(rv) => match &rv.value {
                Some(v) => v.get_type(env),
                None => UVType::Void,
            },
            ASTBlockType::Continue(_) => UVType::Void,
            ASTBlockType::Break(_) => UVType::Void,
        }
    }
}
