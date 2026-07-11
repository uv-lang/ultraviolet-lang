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
            RTVariable::new_from(UVRTValue::Module(evaluator.exports), true),
        );

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }

    /// Parse module export block
    pub fn eval_export(
        &self,
        e: &Vec<SymbolName>,
        env: EnvRef<RTVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        // FIXME: Fix this name resolving
        for exp in e {
            self.exports
                .borrow_mut()
                .define_variable_rc(exp.join("_"), env.borrow().find_var(exp)?);
        }

        Ok(ControlFlow::Simple(UVRTValue::Void))
    }
}
