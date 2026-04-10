use std::slice;

use ultraviolet_core::{
    errors::SpannedError,
    types::frontend::ast::{FunctionCall, FunctionDefinition},
};

use crate::dead_code::{DeadCodeAnalysisFlow, analyze_dead_code};

pub fn analyze_function_def(fd: &FunctionDefinition) -> (Vec<SpannedError>, DeadCodeAnalysisFlow) {
    let (errs, mut t) = analyze_dead_code(&*fd.body);

    // If inner blocks returns a return – catch them
    if matches!(t, DeadCodeAnalysisFlow::Return) {
        t = DeadCodeAnalysisFlow::None;
    }

    (errs, t)
}

pub fn analyze_function_call(fc: &FunctionCall) -> (Vec<SpannedError>, DeadCodeAnalysisFlow) {
    let mut errors = Vec::new();
    let mut term = DeadCodeAnalysisFlow::None;
    for arg in &fc.args {
        let (errs, t) = analyze_dead_code(slice::from_ref(&arg.value));
        errors.extend(errs);
        term = t;

        if !matches!(term, DeadCodeAnalysisFlow::None) {
            break;
        }
    }

    (errors, term)
}
