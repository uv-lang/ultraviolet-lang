use ultraviolet_core::{
    errors::SpannedError,
    traits::{
        EnvironmentTrait,
        frontend::{Positional, UVDisplay},
    },
    types::{
        EnvRef,
        frontend::{
            Spanned,
            ast::{ModuleImport, SymbolName},
            typechecker::{ControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

use crate::typechecker::Typechecker;

impl Typechecker {
    /// typecheck module block
    pub fn typecheck_module(
        &self,
        mi: &Spanned<ModuleImport>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(module_ast) = self.source.modules.get(&mi.name.value) else {
            return Err(SpannedError::new(
                "[INTERNAL ERROR] Cannot find loaded module",
                mi.get_span(),
            ));
        };

        let typechecker = Typechecker::new(module_ast.clone(), &mi.name.value);
        typechecker.start_typecheck()?;

        env.borrow_mut().define_variable(
            typechecker.current_name.clone(),
            UVTypeVariable::new_environmental(typechecker.exports),
        );

        Ok(ControlFlow::Simple(UVType::Void))
    }

    /// Parse module export block
    pub fn typecheck_export(
        &self,
        e: &Vec<SymbolName>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        for exp in e {
            let r = env.borrow().find_var(exp).ok_or(SpannedError::new(
                format!("Variable `{}` for export not defined", exp.join(".")),
                exp.get_span(),
            ))?;

            self.exports
                .borrow_mut()
                .define_variable_rc(exp.join("."), r);
        }

        Ok(ControlFlow::Simple(UVType::Void))
    }
}
