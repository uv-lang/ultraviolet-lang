use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, token_parser::UnwrapOptionError},
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::{
                BuiltInOperation, CompareOpType, ConditionalOperator, LogicalOpType, MathOpType,
            },
            typechecker::{TControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::Typechecker;

fn is_all_comparable(
    a: &[Spanned<UVType>],
    b: &[Spanned<UVType>],
) -> Result<(), (Spanned<UVType>, Spanned<UVType>)> {
    for a_ty in a {
        for b_ty in b {
            let comparable = match (&a_ty.value, &b_ty.value) {
                (UVType::Number(_), UVType::Number(_)) => true,
                (a, b) => a == b,
            };

            if !comparable {
                return Err((a_ty.clone(), b_ty.clone()));
            }
        }
    }

    Ok(())
}

fn all_is_number_like(t: &[Spanned<UVType>]) -> bool {
    t.iter().all(|t| matches!(t.value, UVType::Number(_)))
}

impl Typechecker {
    /// Typecheck math operator
    pub fn check_math_op(
        &self,
        op: &Spanned<BuiltInOperation<MathOpType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = TControlFlow::new_void(op.get_span());
        let first_op_cf = self.typecheck(
            op.operands.first().unwrap_or_spanned(op.get_span())?,
            env.clone(),
        )?;

        let expected_type = UVType::check_all_types(&first_op_cf.ty)
            .map_err(|t| SpannedError::new(format!("Type mismatch: {}", t), t.get_span()))?;

        cf.extend_returns(first_op_cf.returns);
        cf.set_ty(expected_type.clone(), op.get_span());

        for operand in &op.operands {
            let cfa = self.typecheck(operand, env.clone())?;

            expected_type.is_assignable_from_many(&cf.ty).map_err(|t| {
                SpannedError::new(
                    format!(
                        "Type mismatch for math operation: Expected `{}`, got `{}`",
                        expected_type, t
                    ),
                    t.get_span(),
                )
            })?;

            cf.extend_returns(cfa.returns);
        }

        Ok(cf)
    }

    /// Typecheck conditional operator
    pub fn check_conditional_op(
        &self,
        op: &Spanned<ConditionalOperator>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = TControlFlow::new_void(op.get_span());
        let test_cf = self.typecheck(&op.test, env.clone())?;
        cf.extend_returns(test_cf.returns);

        UVType::Boolean
            .is_assignable_from_many(&test_cf.ty)
            .map_err(|t| {
                SpannedError::new(
                    "Conditional operator expects `bool` type for test expression",
                    t.get_span(),
                )
            })?;

        let then_body = match &op.then_body {
            Some(b) => {
                let cfi = self.analyze_group(&b.value, env.clone())?;
                cf.extend_returns(cfi.returns);
                Some(cfi.ty)
            },
            None => None,
        };

        let else_body = match &op.else_body {
            Some(b) => {
                let cfi = self.analyze_group(&b.value, env.clone())?;
                cf.extend_returns(cfi.returns);
                Some(cfi.ty)
            },
            None => None,
        };

        let return_type = match (then_body, else_body) {
            (Some(t), None) => t,
            (None, Some(e)) => e,

            (Some(t), Some(e)) => {
                let mut r = t.clone();
                r.extend(e);
                t
            },

            (None, None) => {
                return Err(SpannedError::new(
                    "Conditional operator cannot be empty",
                    op.get_span(),
                ));
            },
        };

        cf.ty = return_type;
        Ok(cf)
    }

    /// Typecheck logical operator
    pub fn check_logical_op(
        &self,
        op: &Spanned<BuiltInOperation<LogicalOpType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = TControlFlow::new_ty(UVType::Boolean, op.get_span());
        for operand in &op.operands {
            let cfa = self.typecheck(operand, env.clone())?;

            UVType::Boolean
                .is_assignable_from_many(&cfa.ty)
                .map_err(|t| {
                    SpannedError::new(
                        format!(
                            "Logical operator allows only boolean type, but {} provided",
                            t
                        ),
                        t.get_span(),
                    )
                })?;

            cf.extend_returns(cfa.returns);
        }

        Ok(cf)
    }

    /// Typecheck comparison operator
    pub fn check_compare_op(
        &self,
        op: &Spanned<BuiltInOperation<CompareOpType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut types = Vec::new();
        let mut cf = TControlFlow::new_ty(UVType::Boolean, op.get_span());

        for operand in &op.operands {
            let cfi = self.typecheck(operand, env.clone())?;
            types.push(cfi.ty);
            cf.extend_returns(cfi.returns);
        }

        match op.op_type {
            CompareOpType::Equality | CompareOpType::NotEquality => {
                for i in 0..types.len() {
                    for j in (i + 1)..types.len() {
                        is_all_comparable(&types[i], &types[j]).map_err(|(l, r)| {
                            SpannedError::new(
                                format!("Cannot compare `{}` with `{}`", l, r),
                                op.get_span(),
                            )
                        })?;
                    }
                }
            },

            CompareOpType::Greater
            | CompareOpType::GreaterEquals
            | CompareOpType::Less
            | CompareOpType::LessEquals => {
                for t in &types {
                    if !all_is_number_like(t) {
                        return Err(SpannedError::new(
                            format!(
                                "Operator `{}` expects number operands, but `{}` provided",
                                op.op_type,
                                t.first().unwrap()
                            ),
                            op.get_span(),
                        ));
                    }
                }
            },
        }

        Ok(cf)
    }
}
