use std::{ops::Deref, slice};

use ultraviolet_core::{
    errors::{ErrorType, SpannedError},
    traits::frontend::{Positional, ast::GetBlockName},
    types::frontend::ast::ASTBlockType,
};

#[derive(Debug, Clone)]
pub enum DeadCodeAnalysisFlow {
    None,
    Return,
    LoopDiverges,
}

/** Stores and manages the current dead code review thread */
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
        let mut flow_type = upstream.flow_type;
        match node {
            ASTBlockType::FunctionDefinition(_)
                if matches!(flow_type, DeadCodeAnalysisFlow::Return) =>
            {
                flow_type = DeadCodeAnalysisFlow::None
            },
            ASTBlockType::WhileLoop(_) | ASTBlockType::ForLoop(_)
                if matches!(flow_type, DeadCodeAnalysisFlow::LoopDiverges) =>
            {
                flow_type = DeadCodeAnalysisFlow::None
            },
            _ => {},
        }
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
    if let ASTBlockType::Program(p) = ast {
        if let Some(ASTBlockType::HeadBlock(h)) = &p.head {
            errors.extend(analyze_dead_code(&h.value).errors);
        }

        if let ASTBlockType::MainBlock(m) = &p.main {
            errors.extend(analyze_dead_code(&m.value).errors);
        }
    }

    errors
}

// TODO: Сделать нормальный анализ для каждого из типов блоков
/// Analyze code for unreachable elements
pub fn analyze_dead_code<'a>(blocks: impl IntoIterator<Item = &'a ASTBlockType>) -> CheckFlow {
    let mut flow = CheckFlow::new();

    for block in blocks {
        if flow.is_terminates() {
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

        match block {
            ASTBlockType::Return(_) => flow.set_terminal(DeadCodeAnalysisFlow::Return),
            ASTBlockType::Break(_) | ASTBlockType::Continue(_) => {
                flow.set_terminal(DeadCodeAnalysisFlow::LoopDiverges)
            },

            // For simple blocks, just check code and propagates terminal
            ASTBlockType::GroupBlock(inner)
            | ASTBlockType::MainBlock(inner)
            | ASTBlockType::HeadBlock(inner) => {
                flow.apply_flow(analyze_dead_code(inner.deref()), block)
            },

            ASTBlockType::ConditionalOp(cond) => {
                let test_flow = analyze_dead_code(slice::from_ref(&cond.test));

                let then_flow = cond.then_body.as_deref().map(analyze_dead_code);
                let else_flow = cond.else_body.as_deref().map(analyze_dead_code);

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

            ASTBlockType::ForLoop(f) => flow.apply_flow(analyze_dead_code(&*f.body), block),
            ASTBlockType::WhileLoop(w) => flow.apply_flow(analyze_dead_code(&*w.body), block),

            ASTBlockType::FunctionDefinition(f) => {
                flow.apply_flow(analyze_dead_code(&*f.body), block)
            },

            ASTBlockType::FunctionCall(fc) => {
                for arg in &fc.args {
                    let arg_flow = analyze_dead_code(slice::from_ref(&arg.value));
                    let is_term = arg_flow.is_terminates();
                    flow.apply_flow(arg_flow, block);

                    if is_term {
                        break;
                    }
                }
            },

            ASTBlockType::CompareOp(cmp) => {
                for arg in &cmp.operands {
                    let arg_flow = analyze_dead_code(slice::from_ref(arg));
                    let is_term = arg_flow.is_terminates();
                    flow.apply_flow(arg_flow, block);

                    if is_term {
                        break;
                    }
                }
            },

            ASTBlockType::MathOp(cmp) => {
                for arg in &cmp.operands {
                    let arg_flow = analyze_dead_code(slice::from_ref(arg));
                    let is_term = arg_flow.is_terminates();
                    flow.apply_flow(arg_flow, block);

                    if is_term {
                        break;
                    }
                }
            },

            ASTBlockType::LogicalOp(cmp) => {
                for arg in &cmp.operands {
                    let arg_flow = analyze_dead_code(slice::from_ref(arg));
                    let is_term = arg_flow.is_terminates();
                    flow.apply_flow(arg_flow, block);

                    if is_term {
                        break;
                    }
                }
            },

            ASTBlockType::VariableAssignment(assign) => {
                flow.apply_flow(
                    analyze_dead_code(slice::from_ref(assign.value.deref().deref())),
                    block,
                );
            },

            ASTBlockType::VariableDefinition(assign) => {
                flow.apply_flow(
                    analyze_dead_code(slice::from_ref(assign.value.deref())),
                    block,
                );
            },

            _ => {},
        }
    }

    flow
}
