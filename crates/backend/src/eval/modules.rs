use crate::Evaluator;
use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::Positional,
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::{Spanned, ast::ModuleImport},
    },
};

impl Evaluator {
    /// Eval module block
    pub fn eval_module(
        &self,
        mi: &Spanned<ModuleImport>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(module_ast) = self.source.modules.get(&mi.name.value) else {
            return Err(SpannedError::new(
                "[INTERNAL ERROR] Cannot find loaded module",
                mi.get_span(),
            ));
        };

        let evaluator = Evaluator::new(module_ast.clone(), &mi.name.value);
        evaluator.eval()?;

        env.borrow_mut()
            .symbols
            .extend(evaluator.exports.borrow().clone());

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }

    /// Parse module export block
    pub fn eval_export(
        &self,
        e: &Vec<Spanned<String>>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        for exp in e {
            let r = env.borrow().find_var(&exp.value).ok_or(SpannedError::new(
                format!("Variable {} for export not defined", exp.value),
                exp.get_span(),
            ))?;

            self.exports
                .borrow_mut()
                .insert(format!("{}.{}", self.current_name, exp.value.clone()), r);
        }

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }
}
