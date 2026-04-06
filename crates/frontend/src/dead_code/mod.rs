use ultraviolet_core::{
    errors::{ErrorType, SpannedError},
    traits::frontend::{Positional, ast::GetBlockName},
    types::frontend::ast::ASTBlockType,
};

/// Program analyze
pub fn analyze_dead_code_program(ast: &ASTBlockType) -> Vec<SpannedError> {
    let mut errors = Vec::new();
    match ast {
        ASTBlockType::Program(p) => {
            if let Some(head) = &p.head {
                if let ASTBlockType::HeadBlock(h) = head {
                    errors.extend(analyze_dead_code(h));
                }
            }

            if let ASTBlockType::MainBlock(m) = &p.main {
                errors.extend(analyze_dead_code(m));
            }
        },
        _ => {},
    }

    errors
}

/// Analyze code for unreachable elements
pub fn analyze_dead_code(blocks: &[ASTBlockType]) -> Vec<SpannedError> {
    let mut errors = Vec::new();
    let mut found_terminal = false;

    for block in blocks {
        if found_terminal {
            errors.push(
                SpannedError::new(
                    format!(
                        "Found unreachable code during dead code analysis. `{}` is unreachable",
                        block.get_block_name()
                    ),
                    block.get_span(),
                )
                .set_type(ErrorType::Warning),
            )
        }

        match block {
            ASTBlockType::Return(_) | ASTBlockType::Break | ASTBlockType::Continue => {
                found_terminal = true;
            },

            ASTBlockType::GroupBlock(inner)
            | ASTBlockType::MainBlock(inner)
            | ASTBlockType::HeadBlock(inner) => errors.extend(analyze_dead_code(inner)),

            ASTBlockType::ConditionalOp(cond) => {
                if let Some(el) = &cond.then_body {
                    errors.extend(analyze_dead_code(el));
                }
                if let Some(el) = &cond.else_body {
                    errors.extend(analyze_dead_code(el));
                }
            },

            ASTBlockType::ForLoop(f) => {
                errors.extend(analyze_dead_code(&f.body));
            },

            ASTBlockType::WhileLoop(w) => {
                errors.extend(analyze_dead_code(&w.body));
            },

            ASTBlockType::FunctionDefinition(f) => {
                errors.extend(analyze_dead_code(&f.body));
            },

            _ => {},
        }
    }

    errors
}
