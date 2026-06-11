use std::borrow::Cow;

use crate::types::{
    EnvRef,
    frontend::{
        Spanned,
        ast::{ASTBlockType, CompareOpType, LogicalOpType, MathOpType},
        number::UVNumberType,
        typechecker::UVTypeVariable,
        types::UVType,
    },
};

pub trait GetType {
    /// Get type of node / value
    fn get_type(&self) -> UVType;
}

pub trait GetTypeEnv {
    /// Get type of node / value with provided environment
    fn get_type(&self, env: EnvRef<UVTypeVariable>) -> UVType;
}

pub trait IsAssignable {
    /// Returns `true` if `other` is a subtype of `self`.
    ///
    /// This defines assignability in the type system.
    /// A value of type `other` is assignable to `self` if every possible
    /// runtime value of `other` is valid for `self`.
    fn is_assignable_from(&self, other: &UVType) -> bool;
}

pub trait StringToUVType {
    /// Convert string-representation to a Ultraviolet type
    ///
    /// Example:
    /// `String::from("int").to_uvtype();`
    fn to_uvtype(&self) -> Option<UVType>;
}

pub trait StringToUVNumberType {
    /// Convert string-representation to a Ultraviolet Number type
    ///
    /// Example:
    /// `String::from("int").to_uvtype();`
    fn to_uv_number_type(&self) -> Option<UVNumberType>;
}

pub trait StringToUVMathOp {
    /// Convert string-representation to a Ultraviolet math type
    ///
    /// Example:
    /// `String::from("sum").to_uvmath();`
    fn to_uvmath(&self) -> Option<MathOpType>;
}

pub trait StringToUVCompareOp {
    /// Convert string-representation to a Ultraviolet compare op type
    ///
    /// Example:
    /// `String::from("eq").to_uvcompare();`
    fn to_uvcompare(&self) -> Option<CompareOpType>;
}

pub trait StringToUVLogicalOp {
    /// Convert string-representation to a Ultraviolet logical op type
    ///
    /// Example:
    /// `String::from("and").to_uvlogical();`
    fn to_uvlogical(&self) -> Option<LogicalOpType>;
}

pub trait ArgumentsCount {
    /// Get allowed minimum of arguments count
    fn min_arguments_count(&self) -> usize;

    /// Get allowed maximum of arguments count
    fn max_arguments_count(&self) -> Option<usize>;
}

/** Get the name of the current block
*
* Example:
* <let>...</let> will return `let`
*/
pub trait GetBlockName<'a> {
    /// Get name of block
    fn get_block_name(&'a self) -> Cow<'a, str>;
}

pub trait GetOperands {
    /// Return operands for operator
    fn get_operands(&self) -> &Vec<Spanned<ASTBlockType>>;
}
