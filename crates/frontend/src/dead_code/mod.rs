use std::{ops::Deref, slice};

use ultraviolet_core::{
    errors::{ErrorType, SpannedError},
    traits::frontend::{Positional, ast::GetBlockName},
    types::frontend::{Spanned, ast::ASTBlockType},
};

#[derive(Debug, Clone)]
pub enum DeadCodeAnalysisFlow {
    None,
    Return,
    LoopDiverges,
}

/** Stores and manages the current dead code analysis flow */
pub struct CheckFlow {
    pub errors: Vec<SpannedError>,
    pub flow_type: DeadCodeAnalysisFlow,
}

impl CheckFlow {
    /** Create a new check thread */
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            flow_type: DeadCodeAnalysisFlow::None,
        }
    }

    /**
     * Applies an analysis stream to the current stream object.
     *
     * Intercepts the stream type for the corresponding blocks,
     * stopping it from propagates below.
     */
    pub fn apply_flow(&mut self, upstream: Self, node: &ASTBlockType) {
        let flow_type = match (node, upstream.flow_type) {
            (ASTBlockType::FunctionDefinition(_), DeadCodeAnalysisFlow::Return) => {
                DeadCodeAnalysisFlow::None
            },
            (
                ASTBlockType::WhileLoop(_) | ASTBlockType::ForLoop(_),
                DeadCodeAnalysisFlow::LoopDiverges,
            ) => DeadCodeAnalysisFlow::None,
            (_, flow) => flow,
        };
        self.errors.extend(upstream.errors);
        self.flow_type = flow_type;
    }

    /**
     * Applies the found terminal to the flow
     */
    pub fn set_terminal(&mut self, term: DeadCodeAnalysisFlow) {
        self.flow_type = term;
    }

    /** Whether the current flow contains a terminal */
    pub fn is_terminates(&self) -> bool {
        !matches!(self.flow_type, DeadCodeAnalysisFlow::None)
    }

    /** Adds an error to the current flow */
    pub fn add_error(&mut self, error: SpannedError) {
        self.errors.push(error);
    }
}

/// Program analyze
pub fn analyze_dead_code_program(ast: &ASTBlockType) -> Vec<SpannedError> {
    let mut errors = Vec::new();
    if let ASTBlockType::CodeBlock(p) = ast {
        errors.extend(analyze_dead_code(&p.value, false).errors);
    }

    errors
}

/// Analyze code for unreachable elements
pub fn analyze_dead_code<'a>(
    blocks: impl IntoIterator<Item = &'a Spanned<ASTBlockType>>,
    force_terminate: bool,
) -> CheckFlow {
    let mut flow = CheckFlow::new();

    for block in blocks {
        if flow.is_terminates() || force_terminate {
            flow.add_error(
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

        match &**block {
            ASTBlockType::Return(_) => flow.set_terminal(DeadCodeAnalysisFlow::Return),
            ASTBlockType::Break(_) | ASTBlockType::Continue(_) => {
                flow.set_terminal(DeadCodeAnalysisFlow::LoopDiverges)
            },

            // For simple blocks, just check code and propagates terminal
            ASTBlockType::GroupBlock(inner) => {
                flow.apply_flow(analyze_dead_code(inner.deref(), false), block)
            },

            ASTBlockType::ConditionalOp(cond) => {
                let test_flow = analyze_one(&cond.test, false);

                let then_flow = cond
                    .then_body
                    .as_deref()
                    .map(|f| analyze_dead_code(f, false));
                let else_flow = cond
                    .else_body
                    .as_deref()
                    .map(|f| analyze_dead_code(f, false));

                let then_terminates = then_flow.as_ref().is_some_and(|f| f.is_terminates());
                let else_terminates = else_flow.as_ref().is_some_and(|f| f.is_terminates());

                if (test_flow.is_terminates() || (then_terminates && else_terminates))
                    && let Some(f) = then_flow.as_ref().or(else_flow.as_ref())
                {
                    flow.set_terminal(f.flow_type.clone());
                }

                if let Some(f) = then_flow {
                    flow.errors.extend(f.errors);
                }

                if let Some(f) = else_flow {
                    flow.errors.extend(f.errors);
                }
            },

            ASTBlockType::ForLoop(f) => {
                flow.apply_flow(analyze_one(&f.start, false), block);
                flow.apply_flow(analyze_one(&f.end, flow.is_terminates()), block);
                if let Some(step) = &f.step {
                    flow.apply_flow(analyze_one(step, flow.is_terminates()), block);
                }
                flow.apply_flow(analyze_dead_code(&*f.body, flow.is_terminates()), block)
            },

            ASTBlockType::WhileLoop(w) => {
                flow.apply_flow(analyze_one(&w.test, false), block);
                flow.apply_flow(analyze_dead_code(&*w.body, flow.is_terminates()), block)
            },

            ASTBlockType::FunctionDefinition(f) => {
                flow.apply_flow(analyze_dead_code(&*f.body, false), block)
            },

            ASTBlockType::FunctionCall(fc) => analyze_operands(&fc.args, &mut flow, block),
            ASTBlockType::CompareOp(cmp) => analyze_operands(&cmp.operands, &mut flow, block),
            ASTBlockType::MathOp(cmp) => analyze_operands(&cmp.operands, &mut flow, block),
            ASTBlockType::LogicalOp(cmp) => analyze_operands(&cmp.operands, &mut flow, block),

            ASTBlockType::VariableAssignment(assign) => {
                flow.apply_flow(analyze_one(&assign.value.value, false), block);
            },

            ASTBlockType::VariableDefinition(assign) => {
                flow.apply_flow(analyze_one(&assign.value.value, false), block);
            },

            _ => {},
        }
    }

    flow
}

fn analyze_operands<'a>(
    operands: impl IntoIterator<Item = &'a Spanned<ASTBlockType>>,
    flow: &mut CheckFlow,
    block: &ASTBlockType,
) {
    for arg in operands {
        let arg_flow = analyze_one(arg, false);
        let is_term = arg_flow.is_terminates();
        flow.apply_flow(arg_flow, block);

        if is_term {
            break;
        }
    }
}

fn analyze_one(node: &Spanned<ASTBlockType>, force: bool) -> CheckFlow {
    analyze_dead_code(slice::from_ref(node), force)
}
