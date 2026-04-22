use core::fmt;
use std::{borrow::Cow, rc::Rc};

use crate::{
    traits::frontend::{
        Positional,
        ast::{
            ArgumentsCount, GetBlockName, GetType, IsAssignable, StringToUVCompareOp,
            StringToUVLogicalOp, StringToUVMathOp, StringToUVType,
        },
    },
    types::frontend::{Span, Spanned},
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

// ------------------------- Functions ------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UVFunctionType {
    pub args: Vec<UVType>,
    pub returns: UVType,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UVBuiltinFunctionArguments {
    Any,
    Args(Vec<UVType>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UVBuiltinFunctionType {
    pub args: UVBuiltinFunctionArguments,
    pub returns: UVType,
}

// ------------------------------------------------------------------------

/// Ultraviolet number types
///
/// Must be ordered by type width
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UVNumberType {
    Int,
    Float,
}

/// Ultraviolet primitive types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UVType {
    Number(UVNumberType),
    String,
    Boolean,
    Null,
    Void,
    Function(Box<UVFunctionType>),
    BuiltInFunction(Box<UVBuiltinFunctionType>),

    Any,

    Union(Vec<UVType>),
}

impl std::fmt::Display for UVType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UVType::Number(n) => match n {
                UVNumberType::Int => write!(f, "int"),
                UVNumberType::Float => write!(f, "float"),
            },
            UVType::String => write!(f, "str"),
            UVType::Boolean => write!(f, "bool"),
            UVType::Null => write!(f, "null"),
            UVType::Void => write!(f, "void"),
            UVType::Function(_) => write!(f, "<function>"),
            UVType::BuiltInFunction(_) => write!(f, "<built-in function>"),
            UVType::Any => write!(f, "any"),
            UVType::Union(u) => {
                write!(
                    f,
                    "{}",
                    u.iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            },
        }
    }
}

impl UVType {
    /// Create new union type
    pub fn new_union(types: Vec<UVType>) -> UVType {
        let mut flat = Vec::new();

        for t in types {
            t.flatten_into(&mut flat);
        }

        flat.sort();
        flat.dedup();

        if flat.len() == 1 {
            flat.into_iter().next().unwrap()
        } else {
            UVType::Union(flat)
        }
    }

    /// Flat Union type to provided output vector
    pub fn flatten_into(&self, out: &mut Vec<Self>) {
        match self {
            Self::Union(types) => {
                types.iter().for_each(|t| t.flatten_into(out));
            },
            t => out.push(t.clone()),
        }
    }

    /// Get wider number type
    pub fn wider_type(vec: &[UVNumberType]) -> Option<UVNumberType> {
        vec.iter().max().cloned()
    }
}

impl IsAssignable for UVType {
    fn is_assignable_from(&self, other: &UVType) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            (UVType::Number(a), UVType::Number(b)) => b <= a,

            (_, UVType::Union(types)) => types.iter().all(|t| self.is_assignable_from(t)),
            (UVType::Union(types), _) => types.iter().any(|t| t.is_assignable_from(other)),

            (UVType::Any, _) => true,
            (_, UVType::Any) => false,

            _ => false,
        }
    }
}

// -------------------- String-Type conversion --------------

impl StringToUVType for str {
    fn to_uvtype(&self) -> Option<UVType> {
        match self {
            "int" => Some(UVType::Number(UVNumberType::Int)),
            "float" => Some(UVType::Number(UVNumberType::Float)),
            "str" => Some(UVType::String),
            "bool" => Some(UVType::Boolean),
            "null" => Some(UVType::Null),
            "void" => Some(UVType::Void),
            _ => None,
        }
    }
}

// --------------------------- AST-TYPES ---------------------------
#[derive(Debug)]
pub enum ASTBlockType {
    Program(Box<ProgramBlock>),

    HeadBlock(Spanned<Vec<ASTBlockType>>),
    MainBlock(Spanned<Vec<ASTBlockType>>),

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
}

impl<'a> GetBlockName<'a> for ASTBlockType {
    fn get_block_name(&'a self) -> Cow<'a, str> {
        match self {
            ASTBlockType::Program(_) => Cow::Borrowed("program"),
            ASTBlockType::HeadBlock(_) => Cow::Borrowed("head"),
            ASTBlockType::MainBlock(_) => Cow::Borrowed("main"),
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
        }
    }
}

impl Positional for ASTBlockType {
    fn get_span(&self) -> Span {
        match self {
            ASTBlockType::Program(p) => p.span,
            ASTBlockType::HeadBlock(a) => a.span,
            ASTBlockType::MainBlock(a) => a.span,
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
        }
    }
}

// --------------------------- PROGRAM BLOCK ------------------------

#[derive(Debug)]
pub struct ProgramBlock {
    pub head: Option<ASTBlockType>,
    pub main: ASTBlockType,

    pub span: Span,
}

// --------------------------- VariableDefinition BLOCK ------------------------

#[derive(Debug)]
pub struct VariableDefinition {
    pub name: Spanned<String>,
    pub value: Spanned<ASTBlockType>,
    pub expected_type: Option<Spanned<UVType>>,
    pub is_const: bool,

    pub span: Span,
}

// ------------------------- Variable Assign ---------------------------------

#[derive(Debug)]
pub struct VariableAssign {
    pub name: String,
    pub value: Spanned<Box<ASTBlockType>>,

    pub span: Span,
}

// ------------------------ Variable Access ----------------------------------

#[derive(Debug)]
pub struct VariableAccess {
    pub name: String,
    pub span: Span,
}

// ------------------------ Math Operations ----------------------------------
#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
pub struct ForLoop {
    pub iterator: Spanned<String>,
    pub start: ASTBlockType,
    pub end: ASTBlockType,
    pub step: Option<ASTBlockType>,
    pub body: Spanned<Vec<ASTBlockType>>,

    pub span: Span,
}

// ---------------------------- While loop -----------------------------------

#[derive(Debug)]
pub struct WhileLoop {
    pub test: ASTBlockType,
    pub body: Spanned<Vec<ASTBlockType>>,

    pub span: Span,
}

// ---------------------- Conditional Operator -------------------------------

#[derive(Debug)]
pub struct ConditionalOperator {
    pub test: ASTBlockType,
    pub then_body: Option<Spanned<Vec<ASTBlockType>>>,
    pub else_body: Option<Spanned<Vec<ASTBlockType>>>,

    pub span: Span,
}

// ----------------------- Function Definition --------------------------------

#[derive(Debug)]
pub struct FunctionDefinitionArg {
    pub name: Spanned<String>,
    pub arg_type: Spanned<UVType>,

    pub span: Span,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: Spanned<String>,
    pub arguments: Vec<FunctionDefinitionArg>,
    pub return_type: Option<Spanned<UVType>>,

    pub body: Rc<Vec<ASTBlockType>>,

    pub span: Span,
}

// ------------------------- Function Call -----------------------------------
#[derive(Debug)]
pub struct FunctionCallArg {
    pub value: ASTBlockType,

    pub span: Span,
}

#[derive(Debug)]
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
