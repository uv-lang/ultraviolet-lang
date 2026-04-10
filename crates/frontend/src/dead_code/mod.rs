use ultraviolet_core::{
    errors::{ErrorType, SpannedError},
    traits::frontend::{Positional, ast::GetBlockName},
    types::frontend::ast::ASTBlockType,
};

use crate::dead_code::functions::{analyze_function_call, analyze_function_def};

mod functions;

#[derive(Debug)]
pub enum DeadCodeAnalysisFlow {
    None,
    Return,
    LoopDiverges,
}

/// Applies check flow
fn apply_flow(
    errors: &mut Vec<SpannedError>,
    f_t: &mut bool,
    terminal_t: &mut DeadCodeAnalysisFlow,
    (errs, t): (Vec<SpannedError>, DeadCodeAnalysisFlow),
) {
    errors.extend(errs);
    if !matches!(t, DeadCodeAnalysisFlow::None) {
        *f_t = true;
        *terminal_t = t;
    }
}

/// Program analyze
pub fn analyze_dead_code_program(ast: &ASTBlockType) -> Vec<SpannedError> {
    let mut errors = Vec::new();
    if let ASTBlockType::Program(p) = ast {
        if let Some(ASTBlockType::HeadBlock(h)) = &p.head {
            errors.extend(analyze_dead_code(&h.value).0);
        }

        if let ASTBlockType::MainBlock(m) = &p.main {
            errors.extend(analyze_dead_code(&m.value).0);
        }
    }

    errors
}

// TODO: Сделать нормальный анализ для каждого из типов блоков
/// Analyze code for unreachable elements
pub fn analyze_dead_code<'a>(
    blocks: impl IntoIterator<Item = &'a ASTBlockType>,
) -> (Vec<SpannedError>, DeadCodeAnalysisFlow) {
    let mut errors = Vec::new();
    let mut found_terminal = false;
    let mut terminal_type = DeadCodeAnalysisFlow::None;

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
            );
            continue;
        }

        match block {
            ASTBlockType::Return(_) => {
                found_terminal = true;
                terminal_type = DeadCodeAnalysisFlow::Return;
            },
            ASTBlockType::Break(_) | ASTBlockType::Continue(_) => {
                found_terminal = true;
                terminal_type = DeadCodeAnalysisFlow::LoopDiverges;
            },

            // For simple blocks, just check code and propagates terminal
            ASTBlockType::GroupBlock(inner)
            | ASTBlockType::MainBlock(inner)
            | ASTBlockType::HeadBlock(inner) => apply_flow(
                &mut errors,
                &mut found_terminal,
                &mut terminal_type,
                analyze_dead_code(&inner.value),
            ),

            ASTBlockType::ConditionalOp(cond) => {
                let (then_errs, then_flow) = match &cond.then_body {
                    Some(b) => analyze_dead_code(&b.value),
                    None => (vec![], DeadCodeAnalysisFlow::None),
                };

                let (else_errs, else_flow) = match &cond.else_body {
                    Some(b) => analyze_dead_code(&b.value),
                    None => (vec![], DeadCodeAnalysisFlow::None),
                };

                errors.extend(then_errs);
                errors.extend(else_errs);

                if !matches!(then_flow, DeadCodeAnalysisFlow::None)
                    && !matches!(else_flow, DeadCodeAnalysisFlow::None)
                {
                    found_terminal = true;
                    terminal_type = then_flow;
                }
            },

            ASTBlockType::ForLoop(f) => apply_flow(
                &mut errors,
                &mut found_terminal,
                &mut terminal_type,
                analyze_dead_code_loop(&f.body),
            ),

            ASTBlockType::WhileLoop(w) => apply_flow(
                &mut errors,
                &mut found_terminal,
                &mut terminal_type,
                analyze_dead_code_loop(&w.body),
            ),

            ASTBlockType::FunctionDefinition(f) => apply_flow(
                &mut errors,
                &mut found_terminal,
                &mut terminal_type,
                analyze_function_def(&f),
            ),

            ASTBlockType::FunctionCall(fc) => apply_flow(
                &mut errors,
                &mut found_terminal,
                &mut terminal_type,
                analyze_function_call(&fc),
            ),

            _ => {},
        }
    }

    (errors, terminal_type)
}

fn analyze_dead_code_loop(blocks: &[ASTBlockType]) -> (Vec<SpannedError>, DeadCodeAnalysisFlow) {
    let (errs, mut t) = analyze_dead_code(blocks);

    // If inner blocks returns a loop terminator – catch them
    if matches!(t, DeadCodeAnalysisFlow::LoopDiverges) {
        t = DeadCodeAnalysisFlow::None;
    }

    (errs, t)
}
