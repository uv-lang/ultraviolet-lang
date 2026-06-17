use core::fmt;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashSet,
    rc::{Rc, Weak},
};

use crate::{
    traits::{
        UnwrapWeakRefCell,
        frontend::{
            Positional,
            ast::{
                ArgumentsCount, GetBlockName, GetOperands, GetType, StringToUVCompareOp,
                StringToUVLogicalOp, StringToUVMathOp,
            },
        },
    },
    types::frontend::{
        Span, Spanned,
        number::Number,
        types::{ReferenceType, UVType},
    },
};

pub type ASTSpannedBody = Spanned<Vec<Spanned<ASTBlockType>>>;

/// Typed value container
#[derive(Debug, Clone)]
pub enum UVValue {
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Void,

    Reference(Weak<RefCell<UVValue>>),
}

impl GetType for UVValue {
    fn get_type(&self) -> UVType {
        match self {
            UVValue::Number(n) => n.get_type(),
            UVValue::String(_) => UVType::String,
            UVValue::Boolean(_) => UVType::Boolean,
            UVValue::Null => UVType::Null,

            UVValue::Void => UVType::Void,
            // FIXME: Remove unwrap
            // FIXME: Should an internally referenced object be created here?
            UVValue::Reference(r) => UVType::Reference(Box::new(ReferenceType::new(
                r.unwrap_weak().borrow().get_type(),
            ))),
        }
    }
}

impl std::fmt::Display for UVValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UVValue::Number(n) => n.fmt(f),
            UVValue::String(s) => write!(f, "{s}"),
            UVValue::Boolean(b) => write!(f, "{b}"),
            UVValue::Null => write!(f, "null"),
            UVValue::Void => write!(f, "void"),
            // FIXME: Remove unwrap
            UVValue::Reference(r) => write!(f, "{}", r.unwrap_weak().borrow()),
        }
    }
}

// --------------------------- AST-TYPES ---------------------------
pub enum ASTBlockType {
    CodeBlock(ASTSpannedBody),
    ModuleBlock(ASTSpannedBody),

    VariableDefinition(Box<Spanned<VariableDefinition>>),
    FunctionDefinition(Box<Spanned<FunctionDefinition>>),

    FunctionCall(Box<Spanned<FunctionCall>>),
    VariableAssignment(Box<Spanned<VariableAssign>>),
    VariableAccess(Spanned<VariableAccess>),
    ReferenceCreate(Spanned<VariableAccess>),

    ConditionalOp(Box<Spanned<ConditionalOperator>>),

    MathOp(Spanned<BuiltInOperation<MathOpType>>),
    LogicalOp(Spanned<BuiltInOperation<LogicalOpType>>),
    CompareOp(Spanned<BuiltInOperation<CompareOpType>>),

    ForLoop(Box<Spanned<ForLoop>>),
    WhileLoop(Spanned<Box<WhileLoop>>),

    Value(Spanned<UVValue>),

    GroupBlock(ASTSpannedBody),

    Return(Spanned<Option<Box<ASTBlockType>>>),
    Continue(Spanned<()>),
    Break(Spanned<()>),

    FFIDefinition(Spanned<Box<FFIDefinition>>),

    ModuleImport(Spanned<ModuleImport>),
    ModuleExport(Spanned<Vec<Spanned<VariableAccess>>>),
}

impl<'a> GetBlockName<'a> for ASTBlockType {
    fn get_block_name(&'a self) -> Cow<'a, str> {
        match self {
            ASTBlockType::CodeBlock(_) => Cow::Borrowed("code"),
            ASTBlockType::ModuleBlock(_) => Cow::Borrowed("mod"),
            ASTBlockType::VariableDefinition(_) => Cow::Borrowed("let"),
            ASTBlockType::VariableAssignment(a) => Cow::Borrowed(&a.name),
            ASTBlockType::VariableAccess(a) => Cow::Borrowed(&a.name),
            ASTBlockType::ReferenceCreate(r) => Cow::Borrowed(&r.value.name),

            ASTBlockType::MathOp(m) => Cow::Owned(m.op_type.to_string().to_lowercase()),
            ASTBlockType::LogicalOp(l) => Cow::Owned(l.op_type.to_string().to_lowercase()),
            ASTBlockType::CompareOp(c) => Cow::Owned(c.op_type.to_string().to_lowercase()),
            ASTBlockType::Value(v) => Cow::Owned(v.value.to_string().to_lowercase()),

            ASTBlockType::ForLoop(_) => Cow::Borrowed("for"),
            ASTBlockType::WhileLoop(_) => Cow::Borrowed("while"),
            ASTBlockType::Return(_) => Cow::Borrowed("return"),
            ASTBlockType::Continue(_) => Cow::Borrowed("continue"),
            ASTBlockType::Break(_) => Cow::Borrowed("break"),
            ASTBlockType::GroupBlock(_) => Cow::Borrowed("g"),
            ASTBlockType::FunctionDefinition(_) => Cow::Borrowed("fn"),
            ASTBlockType::FunctionCall(_) => Cow::Borrowed("call"),
            ASTBlockType::ConditionalOp(_) => Cow::Borrowed("if"),

            ASTBlockType::ModuleImport(_) => Cow::Borrowed("import"),
            ASTBlockType::ModuleExport(_) => Cow::Borrowed("export"),
            ASTBlockType::FFIDefinition(_) => Cow::Borrowed("ffi"),
        }
    }
}

impl Positional for ASTBlockType {
    fn get_span(&self) -> Span {
        match self {
            ASTBlockType::ModuleBlock(b) => b.get_span(),
            ASTBlockType::CodeBlock(p) => p.get_span(),
            ASTBlockType::VariableDefinition(v) => v.get_span(),
            ASTBlockType::FunctionDefinition(f) => f.get_span(),
            ASTBlockType::FunctionCall(f) => f.get_span(),
            ASTBlockType::VariableAssignment(v) => v.get_span(),
            ASTBlockType::VariableAccess(v) => v.get_span(),
            ASTBlockType::ConditionalOp(c) => c.get_span(),
            ASTBlockType::MathOp(m) => m.get_span(),
            ASTBlockType::LogicalOp(l) => l.get_span(),
            ASTBlockType::CompareOp(c) => c.get_span(),
            ASTBlockType::ForLoop(f) => f.get_span(),
            ASTBlockType::WhileLoop(w) => w.get_span(),
            ASTBlockType::Value(s) => s.get_span(),
            ASTBlockType::GroupBlock(a) => a.get_span(),
            ASTBlockType::Return(a) => a.get_span(),
            ASTBlockType::Continue(c) => c.get_span(),
            ASTBlockType::Break(b) => b.get_span(),
            ASTBlockType::ModuleImport(i) => i.get_span(),
            ASTBlockType::ModuleExport(e) => e.get_span(),
            ASTBlockType::FFIDefinition(f) => f.get_span(),
            ASTBlockType::ReferenceCreate(r) => r.get_span(),
        }
    }
}

// --------------------------- VariableDefinition BLOCK ------------------------

pub struct VariableDefinition {
    pub name: Spanned<String>,
    pub value: Spanned<ASTBlockType>,
    pub expected_type: Option<Spanned<UVType>>,
    pub is_const: bool,
}

// ------------------------- Variable Assign ---------------------------------

pub struct VariableAssign {
    pub name: String,
    pub value: Spanned<ASTBlockType>,
}

// ------------------------ Variable Access ----------------------------------

// FIXME: HA-HA Is this really a structure with one field?
#[derive(Clone, Debug)]
pub struct VariableAccess {
    pub name: String,
}

// ------------------ Generic Operations structure ---------------------------

pub struct BuiltInOperation<T> {
    pub op_type: T,
    pub operands: Vec<Spanned<ASTBlockType>>,
}

impl<T> GetOperands for Spanned<BuiltInOperation<T>> {
    fn get_operands(&self) -> &Vec<Spanned<ASTBlockType>> {
        &self.operands
    }
}

// ------------------------ Math Operations ----------------------------------

#[derive(Debug)]
pub enum MathOpType {
    Sum,
    Sub,
    Mul,
    Div,
    Mod,
}

impl StringToUVMathOp for str {
    fn to_uvmath(&self) -> Option<MathOpType> {
        Some(match self {
            "sum" => MathOpType::Sum,
            "sub" => MathOpType::Sub,
            "mul" => MathOpType::Mul,
            "div" => MathOpType::Div,
            "mod" => MathOpType::Mod,
            _ => return None,
        })
    }
}

impl fmt::Display for MathOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ArgumentsCount for MathOpType {
    fn min_arguments_count(&self) -> usize {
        2
    }

    fn max_arguments_count(&self) -> Option<usize> {
        match self {
            MathOpType::Sum | MathOpType::Mul => None,
            MathOpType::Div | MathOpType::Mod | MathOpType::Sub => Some(2),
        }
    }
}

// ----------------------- Compare Operators ---------------------------------

#[derive(Debug)]
pub enum CompareOpType {
    Equality,
    NotEquality,
    Greater,
    GreaterEquals,
    Less,
    LessEquals,
}

impl fmt::Display for CompareOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ArgumentsCount for CompareOpType {
    fn min_arguments_count(&self) -> usize {
        2
    }

    fn max_arguments_count(&self) -> Option<usize> {
        if matches!(self, Self::Equality) {
            return None;
        }

        Some(2)
    }
}

impl StringToUVCompareOp for str {
    fn to_uvcompare(&self) -> Option<CompareOpType> {
        Some(match self {
            "eq" => CompareOpType::Equality,
            "neq" => CompareOpType::NotEquality,
            "lt" => CompareOpType::Less,
            "lte" => CompareOpType::LessEquals,
            "gt" => CompareOpType::Greater,
            "gte" => CompareOpType::GreaterEquals,
            _ => return None,
        })
    }
}

// ----------------------- Logical Operators ---------------------------------
#[derive(Debug)]
pub enum LogicalOpType {
    And,
    Or,
    Not,
}

impl fmt::Display for LogicalOpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ArgumentsCount for LogicalOpType {
    fn min_arguments_count(&self) -> usize {
        match self {
            LogicalOpType::And | LogicalOpType::Or => 2,
            LogicalOpType::Not => 1,
        }
    }

    fn max_arguments_count(&self) -> Option<usize> {
        match self {
            LogicalOpType::And | LogicalOpType::Or => None,
            LogicalOpType::Not => Some(1),
        }
    }
}

impl StringToUVLogicalOp for str {
    fn to_uvlogical(&self) -> Option<LogicalOpType> {
        Some(match self {
            "and" => LogicalOpType::And,
            "or" => LogicalOpType::Or,
            "not" => LogicalOpType::Not,
            _ => return None,
        })
    }
}

// --------------------------- For loop --------------------------------------
pub struct ForLoop {
    pub iterator: Spanned<String>,
    pub start: Spanned<ASTBlockType>,
    pub end: Spanned<ASTBlockType>,
    pub step: Option<Spanned<ASTBlockType>>,
    pub body: ASTSpannedBody,
}

// ---------------------------- While loop -----------------------------------

pub struct WhileLoop {
    pub test: Spanned<ASTBlockType>,
    pub body: ASTSpannedBody,
}

// ---------------------- Conditional Operator -------------------------------

pub struct ConditionalOperator {
    pub test: Spanned<ASTBlockType>,
    pub then_body: Option<ASTSpannedBody>,
    pub else_body: Option<ASTSpannedBody>,
}

// ----------------------- Function Definition --------------------------------

pub struct FunctionDefinitionArg {
    pub name: Spanned<String>,
    pub arg_type: Spanned<UVType>,
}

pub struct FunctionDefinition {
    pub name: Option<Spanned<String>>,
    pub arguments: Vec<Spanned<FunctionDefinitionArg>>,
    pub return_type: Option<Spanned<UVType>>,

    pub body: Rc<Vec<Spanned<ASTBlockType>>>,
    pub moved_symbols: RefCell<HashSet<String>>,
}

// ------------------------- Function Call -----------------------------------

pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Spanned<ASTBlockType>>,
}

// ------------------------- FFI Definition ----------------------------------

/// Definition of FFI function
pub struct FFIDefinition {
    pub name: Spanned<String>,
    pub dll: Spanned<ASTBlockType>,
    pub func: Spanned<ASTBlockType>,

    pub arguments: Vec<Spanned<UVType>>,
    pub return_type: Option<Spanned<UVType>>,
}

// ---------------------------- Modules ----------------------------------------

/// Represents a module, that should be imported
#[derive(Clone, Debug)]
pub struct ModuleImport {
    pub path: Spanned<String>,
    pub name: Spanned<String>,
}
// ---------------------------- TESTS ----------------------------------------

#[cfg(test)]
mod tests {
    use crate::{
        traits::frontend::ast::{IsAssignable, StringToUVType},
        types::frontend::{ast::UVType, number::UVNumberType},
    };

    #[test]
    fn parse_type() {
        assert_eq!(
            String::from("i32").to_uvtype(),
            Some(UVType::Number(UVNumberType::I32))
        );
        assert_eq!(String::from("bool").to_uvtype(), Some(UVType::Boolean));
        assert_eq!(
            String::from("f64").to_uvtype(),
            Some(UVType::Number(UVNumberType::F64))
        );
        assert_eq!(String::from("null").to_uvtype(), Some(UVType::Null));
        assert_eq!(String::from("str").to_uvtype(), Some(UVType::String));

        assert_eq!(String::from("unknown").to_uvtype(), None);
    }

    #[test]
    fn type_compatible_with() {
        assert!(!UVType::Number(UVNumberType::I64).is_assignable_from(&UVType::Boolean));
    }
}
