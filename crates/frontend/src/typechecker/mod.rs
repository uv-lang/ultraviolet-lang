use std::rc::Rc;

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::{Positional, ast::GetType},
    types::{
        EnvRef, Environment,
        builtins::DefineBuiltinsType,
        frontend::{
            SourceFileParsed, Span, Spanned,
            ast::{ASTBlockType, AccessType, AssignType},
            typechecker::{TControlFlow, UVTypeVariable},
            types::UVType,
        },
    },
};

mod ffi;
mod functions;
mod loops;
mod modules;
mod namespace;
mod operators;
mod variables;

pub struct Typechecker {
    pub source: Rc<SourceFileParsed>,
    pub current_name: String,
    pub exports: EnvRef<UVTypeVariable>,
}

impl Typechecker {
    pub fn new(sf: Rc<SourceFileParsed>, name: impl Into<String>) -> Self {
        Self {
            source: sf,
            current_name: name.into(),
            exports: Environment::new(),
        }
    }

    pub fn start_typecheck(&self) -> Result<(), SpannedError> {
        let env = Environment::<UVTypeVariable>::new();
        env.define_builtins();
        self.typecheck(&self.source.ast, env)?;

        Ok(())
    }

    /// Recursive typecheck
    fn typecheck(
        &self,
        block: &ASTBlockType,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        Ok(match block {
            ASTBlockType::CodeBlock(m) | ASTBlockType::ModuleBlock(m) => {
                self.analyze_group(&m, env)?
            },

            ASTBlockType::VariableDefinition(vd) => self.check_variable_definition(vd, env)?,
            ASTBlockType::VariableAssignment(va) => match va.assign_type {
                AssignType::Simple => self.check_variable_assign(va, env)?,
                AssignType::Dereference => self.check_dereference_assign(va, env)?,
            },

            ASTBlockType::VariableAccess(va) => match va.access_type {
                AccessType::Simple => self.check_variable_access(va, env)?,
                AccessType::Dereference => self.check_dereference(va, env)?,
                AccessType::Reference => self.check_reference_create(va, env)?,
            },

            ASTBlockType::FunctionDefinition(fd) => self.check_function_definition(fd, env)?,
            ASTBlockType::FunctionCall(fc) => self.check_function_call(fc, env)?,

            ASTBlockType::ConditionalOp(co) => self.check_conditional_op(co, env)?,
            ASTBlockType::MathOp(mo) => self.check_math_op(mo, env)?,
            ASTBlockType::LogicalOp(lo) => self.check_logical_op(lo, env)?,
            ASTBlockType::CompareOp(co) => self.check_compare_op(co, env)?,

            ASTBlockType::ForLoop(fl) => self.check_for_loop(fl, env)?,
            ASTBlockType::WhileLoop(wl) => self.check_while_loop(wl, env)?,

            ASTBlockType::Value(v) => TControlFlow::new_ty(v.get_type(), v.get_span()),
            ASTBlockType::GroupBlock(g) => self.analyze_group(&g, env)?,
            ASTBlockType::Return(r) => self.analyze_return(r, env)?,
            ASTBlockType::Continue(s) | ASTBlockType::Break(s) => {
                // FIXME: Should it be never?
                TControlFlow::new_ty(UVType::Never, s.get_span())
            },

            ASTBlockType::FFIDefinition(ffi_d) => self.check_ffi_definition(ffi_d, env)?,

            ASTBlockType::ModuleImport(i) => self.typecheck_module(i, env)?,
            ASTBlockType::ModuleExport(e) => self.typecheck_export(e, env)?,
            ASTBlockType::Namespace(ns) => self.typecheck_namespace(ns, env)?,
        })
    }

    /// Analyze group of block
    ///
    /// Handle return and passes upstream
    /// Returns latest block type as group type
    fn analyze_group(
        &self,
        blocks: &Vec<Spanned<ASTBlockType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let new_env = Environment::new_child(env);

        let mut cf = TControlFlow::new_void(Span::new(0, 0, self.source.source.clone()));
        for node in blocks {
            let cfi = self.typecheck(node, new_env.clone())?;
            cf.extend_from(cfi);
        }

        Ok(cf)
    }

    /// Analyze return block
    fn analyze_return(
        &self,
        ret: &Spanned<Option<Box<ASTBlockType>>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<TControlFlow, SpannedError> {
        let mut cf = TControlFlow::new_ty(UVType::Never, ret.get_span());
        let Some(b) = &ret.value else {
            cf.add_returns(Spanned::new(UVType::Void, ret.get_span()));
            return Ok(cf);
        };

        let cfi = self.typecheck(&b, env)?;
        cf.extend_returns(cfi.ty);
        cf.returns.extend(cfi.returns);
        Ok(cf)
    }
}
