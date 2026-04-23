use ultraviolet_core::{
    errors::SpannedError,
    types::{
        EnvRef, Environment,
        frontend::{
            ast::{CompareOp, CompareOpType, ConditionalOperator, LogicalOp, MathOp},
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::{analyze_group, typecheck};

/// Typecheck math operator
pub fn check_math_op(
    op: &MathOp,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let mut op_types = Vec::new();
    for operand in &op.operands {
        let t = match typecheck(operand, env.clone())? {
            ControlFlow::Simple(t) => t,
            cf => return Ok(cf),
        };

        let number_type = match t {
            UVType::Number(n) => n,
            other => {
                return Err(SpannedError::new(
                    format!("Math operator expects number, but `{}` provided", other),
                    op.span,
                ));
            },
        };

        op_types.push(number_type);
    }

    let wider = UVType::wider_type(&op_types).ok_or_else(|| {
        SpannedError::new(
            "Math operator requires at least one operand".to_string(),
            op.span,
        )
    })?;

    Ok(ControlFlow::Simple(UVType::Number(wider)))
}

/// Typecheck conditional operator
pub fn check_conditional_op(
    op: &ConditionalOperator,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let test = match typecheck(&op.test, env.clone())? {
        ControlFlow::Simple(t) => t,
        cf => return Ok(cf),
    };

    if !matches!(test, UVType::Boolean) {
        return Err(SpannedError::new(
            "Conditional operator expects `bool` type for test expression",
            op.span,
        ));
    }

    let then_body = match &op.then_body {
        Some(b) => Some(
            match analyze_group(&b.value, Environment::new_child(env.clone()))? {
                ControlFlow::Simple(t) => t,
                // TODO: Make it so that it doesn’t skip checking other branches
                cf => return Ok(cf),
            },
        ),
        None => None,
    };

    let else_body = match &op.else_body {
        Some(b) => Some(
            match analyze_group(&b.value, Environment::new_child(env.clone()))? {
                ControlFlow::Simple(t) => t,
                cf => return Ok(cf),
            },
        ),
        None => None,
    };

    let return_type = match (then_body, else_body) {
        (None, None) => UVType::Void,
        (None, Some(t)) | (Some(t), None) => UVType::new_union(vec![t, UVType::Void]),
        (Some(t), Some(_)) => t,
    };

    Ok(ControlFlow::Simple(return_type))
}

/// Typecheck logical operator
pub fn check_logical_op(
    op: &LogicalOp,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    for operand in &op.operands {
        let t = match typecheck(operand, env.clone())? {
            ControlFlow::Simple(t) => t,
            cf => return Ok(cf),
        };

        if !matches!(t, UVType::Boolean) {
            return Err(SpannedError::new(
                format!(
                    "Logical operator allows only boolean type, but {} provided",
                    t
                ),
                op.span,
            ));
        }
    }

    Ok(ControlFlow::Simple(UVType::Boolean))
}

fn are_comparable(a: &UVType, b: &UVType) -> bool {
    match (a, b) {
        (UVType::Number(_), UVType::Number(_)) => true,

        _ if a == b => true,

        (UVType::Union(types), other) => types.iter().all(|t| are_comparable(t, other)),
        (other, UVType::Union(types)) => types.iter().all(|t| are_comparable(other, t)),

        _ => false,
    }
}

fn is_number_like(t: &UVType) -> bool {
    match t {
        UVType::Number(_) => true,
        UVType::Union(types) => types.iter().all(is_number_like),
        _ => false,
    }
}

/// Typecheck comparison operator
pub fn check_compare_op(
    op: &CompareOp,
    env: EnvRef<UVTypeVariable>,
) -> Result<ControlFlow, SpannedError> {
    let mut types = Vec::new();

    for operand in &op.operands {
        let t = match typecheck(operand, env.clone())? {
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
                            op.span,
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
                        op.span,
                    ));
                }
            }
        },
    }

    Ok(ControlFlow::Simple(UVType::Boolean))
}
