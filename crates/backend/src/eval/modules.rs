use crate::Evaluator;
use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, UVDisplay},
    },
    types::{
        EnvRef,
        backend::{ControlFlow, RTVariable, UVRTValue},
        frontend::{
            Spanned,
            ast::{ModuleImport, SymbolName},
        },
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

        env.borrow_mut().define_variable(
            evaluator.current_name.clone(),
            RTVariable::new_environmental(evaluator.exports),
        );

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }

    /// Parse module export block
    pub fn eval_export(
        &self,
        e: &Vec<SymbolName>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        for exp in e {
            let r = env.borrow().find_var(exp).ok_or(SpannedError::new(
                format!("Variable {} for export not defined", exp.join(".")),
                exp.get_span(),
            ))?;

            self.exports
                .borrow_mut()
                .define_variable_rc(exp.join("."), r);
        }

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }
}
