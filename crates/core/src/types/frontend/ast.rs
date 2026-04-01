use crate::{
    traits::frontend::ast::{
        ArgumentsCount, GetType, IsAssignable, StringToUVCompareOp, StringToUVLogicalOp, StringToUVMathOp,
        StringToUVType,
    },
    types::frontend::{Span, Spanned},
};

/// Typed value container
#[derive(Debug, Clone)]
pub enum UVValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,

    Void,
}

impl GetType for UVValue {
    fn get_type(&self) -> UVType {
        match self {
            UVValue::Int(_) => UVType::Int,
            UVValue::Float(_) => UVType::Float,
            UVValue::String(_) => UVType::String,
            UVValue::Boolean(_) => UVType::Boolean,
            UVValue::Null => UVType::Null,

            UVValue::Void => UVType::Void,
        }
    }
}

impl std::fmt::Display for UVValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UVValue::Int(i) => write!(f, "{}", i),
            UVValue::Float(fl) => write!(f, "{}", fl),
            UVValue::String(s) => write!(f, "{}", s),
            UVValue::Boolean(b) => write!(f, "{}", b),
            UVValue::Null => write!(f, "null"),
            UVValue::Void => write!(f, "void"),
        }
    }
}

/// Ultraviolet primitive types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UVType {
    Int,
    Float,
    String,
    Boolean,
    Null,
    Void,

    Union(Vec<UVType>),
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
}

impl IsAssignable for UVType {
    fn is_assignable_from(&self, other: &UVType) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            (_, UVType::Union(types)) => types.iter().all(|t| self.is_assignable_from(t)),
            (UVType::Union(types), _) => types.iter().any(|t| t.is_assignable_from(other)),

            _ => false,
        }
    }
}

// -------------------- String-Type conversion --------------

impl StringToUVType for str {
    fn to_uvtype(&self) -> Option<UVType> {
        match self {
            "int" => Some(UVType::Int),
            "float" => Some(UVType::Float),
            "str" => Some(UVType::String),
            "bool" => Some(UVType::Boolean),
            "null" => Some(UVType::Null),
            "void" => Some(UVType::Void),
            _ => None,
        }
    }
}

// ---------------
/*
#[derive(Debug)]
pub enum Symbol {
    /// Primitive type
    Primitive(UVValue),

    /// Name of the variable in scope
    Variable(String),
}

impl GetTypeScope for Symbol {
    fn get_type_from_scope(&self, scope: Option<usize>) -> UVType {
        match self {
            Self::Primitive(val) => val.get_type(),
            // Scope-based search of the final primitive
            Self::Variable(var) => todo!(),
        }
    }
}
*/

// --------------------------- AST-TYPES ---------------------------
#[derive(Debug)]
pub enum ASTBlockType {
    Program(Box<ProgramBlock>),

    HeadBlock(Vec<ASTBlockType>),
    MainBlock(Vec<ASTBlockType>),

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

    GroupBlock(Box<Vec<ASTBlockType>>),

    Return(Box<ASTBlockType>),
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

impl GetType for VariableDefinition {
    fn get_type(&self) -> UVType {
        todo!()
    }
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

    pub body: Vec<ASTBlockType>,

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
    use crate::{
        traits::frontend::ast::{IsAssignable, StringToUVType},
        types::frontend::ast::UVType,
    };

    #[test]
    fn parse_type() {
        assert_eq!(String::from("int").to_uvtype(), Some(UVType::Int));
        assert_eq!(String::from("bool").to_uvtype(), Some(UVType::Boolean));
        assert_eq!(String::from("float").to_uvtype(), Some(UVType::Float));
        assert_eq!(String::from("null").to_uvtype(), Some(UVType::Null));
        assert_eq!(String::from("str").to_uvtype(), Some(UVType::String));

        assert_eq!(String::from("unknown").to_uvtype(), None);
    }

    #[test]
    fn type_compatible_with() {
        assert!(UVType::Union(vec![UVType::Int, UVType::Null]).is_assignable_from(&UVType::Null));

        assert!(UVType::Union(vec![UVType::Int, UVType::Float]).is_assignable_from(&UVType::Union(vec![UVType::Int])));

        assert!(!UVType::Int.is_assignable_from(&UVType::Union(vec![UVType::Int, UVType::Null])));

        assert!(!UVType::Int.is_assignable_from(&UVType::Boolean));
    }
}
