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
            typechecker::{TControlFlow, UVTypeVariable},
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
    ) -> Result<TControlFlow, SpannedError> {
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
            UVTypeVariable::new_from(UVType::Module(typechecker.exports), true),
        );

        Ok(TControlFlow::new_void(mi.get_span()))
    }

    /// Parse module export block
    pub fn typecheck_export(
        &self,
        e: &Spanned<Vec<SymbolName>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        // FIXME: Fix this name resolving
        for exp in &e.value {
            self.exports
                .borrow_mut()
                .define_variable_rc(exp.join("_"), env.borrow().find_var(exp)?);
        }

        Ok(TControlFlow::new_void(e.get_span()))
    }
}
