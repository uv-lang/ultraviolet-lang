use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::IsAssignable, token_parser::UnwrapOptionError},
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::{
                BuiltInOperation, CompareOpType, ConditionalOperator, LogicalOpType, MathOpType,
            },
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::Typechecker;

fn are_comparable(a: &UVType, b: &UVType) -> bool {
    match (a, b) {
        (UVType::Number(_), UVType::Number(_)) => true,

        _ if a == b => true,
        _ => false,
    }
}

fn is_number_like(t: &UVType) -> bool {
    matches!(t, UVType::Number(_))
}

impl Typechecker {
    /// Typecheck math operator
    pub fn check_math_op(
        &self,
        op: &Spanned<BuiltInOperation<MathOpType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let op_type = match self.typecheck(
            op.operands.first().unwrap_or_spanned(op.get_span())?,
            env.clone(),
        )? {
            ControlFlow::Simple(t) => t,
            cf => return Ok(cf),
        };

        for operand in &op.operands {
            let t = match self.typecheck(operand, env.clone())? {
                ControlFlow::Simple(t) => t,
                cf => return Ok(cf),
            };

            if !op_type.is_assignable_from(&t) {
                return Err(SpannedError::new(
                    format!(
                        "Type mismatch for math operation: Expected `{}`, got `{}`",
                        op_type, t
                    ),
                    operand.get_span(),
                ));
            }
        }

        Ok(ControlFlow::Simple(op_type))
    }

    /// Typecheck conditional operator
    pub fn check_conditional_op(
        &self,
        op: &Spanned<ConditionalOperator>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let test = match self.typecheck(&op.test, env.clone())? {
            ControlFlow::Simple(t) => t,
            cf => return Ok(cf),
        };

        if !matches!(test, UVType::Boolean) {
            return Err(SpannedError::new(
                "Conditional operator expects `bool` type for test expression",
                op.get_span(),
            ));
        }

        let then_body = match &op.then_body {
            Some(b) => Some(self.analyze_group(&b.value, env.clone())?),
            None => None,
        };

        let else_body = match &op.else_body {
            Some(b) => Some(self.analyze_group(&b.value, env.clone())?),
            None => None,
        };

        let return_type = match (then_body, else_body) {
            (None, None) => UVType::Void,
            // Both hands is simple expr
            (Some(ControlFlow::Simple(l)), Some(ControlFlow::Simple(r))) if l == r => l,
            (Some(l), Some(r)) if r == l => l,
            _ => {
                return Err(SpannedError::new(
                    "Type mismatch for conditional operator hands",
                    op.get_span(),
                ));
            },
        };

        Ok(ControlFlow::Simple(return_type))
    }

    /// Typecheck logical operator
    pub fn check_logical_op(
        &self,
        op: &Spanned<BuiltInOperation<LogicalOpType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        for operand in &op.operands {
            let t = match self.typecheck(operand, env.clone())? {
                ControlFlow::Simple(t) => t,
                cf => return Ok(cf),
            };

            if !matches!(t, UVType::Boolean) {
                return Err(SpannedError::new(
                    format!(
                        "Logical operator allows only boolean type, but {} provided",
                        t
                    ),
                    op.get_span(),
                ));
            }
        }

        Ok(ControlFlow::Simple(UVType::Boolean))
    }

    /// Typecheck comparison operator
    pub fn check_compare_op(
        &self,
        op: &Spanned<BuiltInOperation<CompareOpType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let mut types = Vec::new();

        for operand in &op.operands {
            let t = match self.typecheck(operand, env.clone())? {
                ControlFlow::Simple(t) => t,
                cf => return Ok(cf),
            };

            types.push(t);
        }

        match op.op_type {
            CompareOpType::Equality | CompareOpType::NotEquality => {
                for i in 0..types.len() {
                    for j in (i + 1)..types.len() {
                        if !are_comparable(&types[i], &types[j]) {
                            return Err(SpannedError::new(
                                format!("Cannot compare `{}` with `{}`", types[i], types[j]),
                                op.get_span(),
                            ));
                        }
                    }
                }
            },

            CompareOpType::Greater
            | CompareOpType::GreaterEquals
            | CompareOpType::Less
            | CompareOpType::LessEquals => {
                for t in &types {
                    if !is_number_like(t) {
                        return Err(SpannedError::new(
                            format!(
                                "Operator `{}` expects number operands, but `{}` provided",
                                op.op_type, t
                            ),
                            op.get_span(),
                        ));
                    }
                }
            },
        }

        Ok(ControlFlow::Simple(UVType::Boolean))
    }
}
