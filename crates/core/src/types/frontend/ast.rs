use core::fmt;
use std::{borrow::Cow, rc::Rc};

use crate::{
    traits::frontend::{
        Positional,
        ast::{
            ArgumentsCount, GetBlockName, GetType, StringToUVCompareOp, StringToUVLogicalOp,
            StringToUVMathOp,
        },
    },
    types::frontend::{
        ModuleImport, Span, Spanned,
        types::{UVNumberType, UVType},
    },
};

/// Number-like value
#[derive(Debug, Clone)]
pub enum Number {
    Int(i64),
    Float(f64),
}

/// Typed value container
#[derive(Debug, Clone)]
pub enum UVValue {
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
    Void,
}

impl GetType for Number {
    fn get_type(&self) -> UVType {
        match self {
            Number::Int(_) => UVType::Number(UVNumberType::Int),
            Number::Float(_) => UVType::Number(UVNumberType::Float),
        }
    }
}

impl GetType for UVValue {
    fn get_type(&self) -> UVType {
        match self {
            UVValue::Number(n) => n.get_type(),
            UVValue::String(_) => UVType::String,
            UVValue::Boolean(_) => UVType::Boolean,
            UVValue::Null => UVType::Null,

            UVValue::Void => UVType::Void,
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(i) => write!(f, "{i}"),
            Number::Float(fl) => write!(f, "{fl}"),
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
        }
    }
}

// --------------------------- AST-TYPES ---------------------------
pub enum ASTBlockType {
    CodeBlock(Spanned<Vec<ASTBlockType>>),

    VariableDefinition(Box<VariableDefinition>),
    FunctionDefinition(Box<FunctionDefinition>),

    FunctionCall(FunctionCall),
    VariableAssignment(VariableAssign),
    VariableAccess(VariableAccess),

    ConditionalOp(Box<ConditionalOperator>),

    MathOp(MathOp),
    LogicalOp(LogicalOp),
    CompareOp(CompareOp),

    ForLoop(Box<ForLoop>),
    WhileLoop(Box<WhileLoop>),

    Value(Spanned<UVValue>),

    GroupBlock(Spanned<Vec<ASTBlockType>>),

    Return(Spanned<Option<Box<ASTBlockType>>>),
    Continue(Spanned<()>),
    Break(Spanned<()>),

    ModuleImport(Spanned<ModuleImport>),
}

impl<'a> GetBlockName<'a> for ASTBlockType {
    fn get_block_name(&'a self) -> Cow<'a, str> {
        match self {
            ASTBlockType::CodeBlock(_) => Cow::Borrowed("code"),
            ASTBlockType::VariableDefinition(_) => Cow::Borrowed("let"),
            ASTBlockType::VariableAssignment(a) => Cow::Borrowed(&a.name),
            ASTBlockType::VariableAccess(a) => Cow::Borrowed(&a.name),

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
        }
    }
}

impl Positional for ASTBlockType {
    fn get_span(&self) -> Span {
        match self {
            ASTBlockType::CodeBlock(p) => p.span,
            ASTBlockType::VariableDefinition(v) => v.span,
            ASTBlockType::FunctionDefinition(f) => f.span,
            ASTBlockType::FunctionCall(f) => f.span,
            ASTBlockType::VariableAssignment(v) => v.span,
            ASTBlockType::VariableAccess(v) => v.span,
            ASTBlockType::ConditionalOp(c) => c.span,
            ASTBlockType::MathOp(m) => m.span,
            ASTBlockType::LogicalOp(l) => l.span,
            ASTBlockType::CompareOp(c) => c.span,
            ASTBlockType::ForLoop(f) => f.span,
            ASTBlockType::WhileLoop(w) => w.span,
            ASTBlockType::Value(s) => s.span,
            ASTBlockType::GroupBlock(a) => a.span,
            ASTBlockType::Return(a) => a.span,
            ASTBlockType::Continue(c) => c.span,
            ASTBlockType::Break(b) => b.span,
            ASTBlockType::ModuleImport(i) => i.span,
        }
    }
}

// --------------------------- VariableDefinition BLOCK ------------------------

pub struct VariableDefinition {
    pub name: Spanned<String>,
    pub value: Spanned<ASTBlockType>,
    pub expected_type: Option<Spanned<UVType>>,
    pub is_const: bool,

    pub span: Span,
}

// ------------------------- Variable Assign ---------------------------------

pub struct VariableAssign {
    pub name: String,
    pub value: Spanned<Box<ASTBlockType>>,

    pub span: Span,
}

// ------------------------ Variable Access ----------------------------------

pub struct VariableAccess {
    pub name: String,
    pub span: Span,
}

// ------------------------ Math Operations ----------------------------------
pub struct MathOp {
    pub op_type: MathOpType,
    pub operands: Vec<ASTBlockType>,
    pub span: Span,
}

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

pub struct CompareOp {
    pub op_type: CompareOpType,
    pub operands: Vec<ASTBlockType>,
    pub span: Span,
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

pub struct LogicalOp {
    pub op_type: LogicalOpType,
    pub operands: Vec<ASTBlockType>,
    pub span: Span,
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
    pub start: ASTBlockType,
    pub end: ASTBlockType,
    pub step: Option<ASTBlockType>,
    pub body: Spanned<Vec<ASTBlockType>>,

    pub span: Span,
}

// ---------------------------- While loop -----------------------------------

pub struct WhileLoop {
    pub test: ASTBlockType,
    pub body: Spanned<Vec<ASTBlockType>>,

    pub span: Span,
}

// ---------------------- Conditional Operator -------------------------------

pub struct ConditionalOperator {
    pub test: ASTBlockType,
    pub then_body: Option<Spanned<Vec<ASTBlockType>>>,
    pub else_body: Option<Spanned<Vec<ASTBlockType>>>,

    pub span: Span,
}

// ----------------------- Function Definition --------------------------------

pub struct FunctionDefinitionArg {
    pub name: Spanned<String>,
    pub arg_type: Spanned<UVType>,

    pub span: Span,
}

pub struct FunctionDefinition {
    pub name: Option<Spanned<String>>,
    pub arguments: Vec<FunctionDefinitionArg>,
    pub return_type: Option<Spanned<UVType>>,

    pub body: Rc<Vec<ASTBlockType>>,

    pub span: Span,
}

// ------------------------- Function Call -----------------------------------
pub struct FunctionCallArg {
    pub value: ASTBlockType,

    pub span: Span,
}

pub struct FunctionCall {
    pub name: String,
    pub args: Vec<FunctionCallArg>,

    pub span: Span,
}
// ---------------------------- TESTS ----------------------------------------

#[cfg(test)]
mod tests {
    use crate::types::frontend::ast::UVNumberType;
    use crate::{
        traits::frontend::ast::{IsAssignable, StringToUVType},
        types::frontend::ast::UVType,
    };

    #[test]
    fn parse_type() {
        assert_eq!(
            String::from("int").to_uvtype(),
            Some(UVType::Number(UVNumberType::Int))
        );
        assert_eq!(String::from("bool").to_uvtype(), Some(UVType::Boolean));
        assert_eq!(
            String::from("float").to_uvtype(),
            Some(UVType::Number(UVNumberType::Float))
        );
        assert_eq!(String::from("null").to_uvtype(), Some(UVType::Null));
        assert_eq!(String::from("str").to_uvtype(), Some(UVType::String));

        assert_eq!(String::from("unknown").to_uvtype(), None);
    }

    #[test]
    fn type_compatible_with() {
        assert!(
            UVType::Union(vec![UVType::Number(UVNumberType::Float), UVType::Null])
                .is_assignable_from(&UVType::Null)
        );

        assert!(
            UVType::Union(vec![
                UVType::Number(UVNumberType::Int),
                UVType::Number(UVNumberType::Float)
            ])
            .is_assignable_from(&UVType::Union(vec![UVType::Number(UVNumberType::Int)]))
        );

        assert!(
            !UVType::Number(UVNumberType::Int).is_assignable_from(&UVType::Union(vec![
                UVType::Number(UVNumberType::Int),
                UVType::Null
            ]))
        );

        assert!(!UVType::Number(UVNumberType::Int).is_assignable_from(&UVType::Boolean));
    }
}
