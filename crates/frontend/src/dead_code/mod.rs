use ultraviolet_core::{
    errors::{ErrorType, SpannedError},
    traits::frontend::{Positional, ast::GetBlockName},
    types::frontend::ast::ASTBlockType,
};

pub enum DeadCodeAnalysisFlow {
    None,
    Return,
    LoopDiverges,
}

/// Program analyze
pub fn analyze_dead_code_program(ast: &ASTBlockType) -> Vec<SpannedError> {
    let mut errors = Vec::new();
    if let ASTBlockType::Program(p) = ast {
        if let Some(ASTBlockType::HeadBlock(h)) = &p.head {
            errors.extend(analyze_dead_code(h).0);
        }

        if let ASTBlockType::MainBlock(m) = &p.main {
            errors.extend(analyze_dead_code(m).0);
        }
    }

    errors
}

/// Analyze code for unreachable elements
pub fn analyze_dead_code(blocks: &[ASTBlockType]) -> (Vec<SpannedError>, DeadCodeAnalysisFlow) {
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
            ASTBlockType::Break | ASTBlockType::Continue => {
                found_terminal = true;
                terminal_type = DeadCodeAnalysisFlow::LoopDiverges;
            },

            // For simple blocks, just check code and propagates terminal
            ASTBlockType::GroupBlock(inner)
            | ASTBlockType::MainBlock(inner)
            | ASTBlockType::HeadBlock(inner) => {
                let (errs, t) = analyze_dead_code(inner);
                errors.extend(errs);
                terminal_type = t;
            },

            ASTBlockType::ConditionalOp(cond) => {
                let (then_errs, then_flow) = match &cond.then_body {
                    Some(b) => analyze_dead_code(b),
                    None => (vec![], DeadCodeAnalysisFlow::None),
                };

                let (else_errs, else_flow) = match &cond.else_body {
                    Some(b) => analyze_dead_code(b),
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

            ASTBlockType::ForLoop(f) => {
                let (errs, t) = analyze_dead_code_loop(&f.body);
                errors.extend(errs);
                terminal_type = t;
            },

            ASTBlockType::WhileLoop(w) => {
                let (errs, t) = analyze_dead_code_loop(&w.body);
                errors.extend(errs);
                terminal_type = t;
            },

            ASTBlockType::FunctionDefinition(f) => {
                let (errs, t) = analyze_dead_code_function_def(&f.body);
                errors.extend(errs);
                terminal_type = t;
            },

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

fn analyze_dead_code_function_def(
    blocks: &[ASTBlockType],
) -> (Vec<SpannedError>, DeadCodeAnalysisFlow) {
    let (errs, mut t) = analyze_dead_code(blocks);

    // If inner blocks returns a return – catch them
    if matches!(t, DeadCodeAnalysisFlow::Return) {
        t = DeadCodeAnalysisFlow::None;
    }

    (errs, t)
}
