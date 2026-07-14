use std::rc::Rc;

use ultraviolet_core::{
    errors::SpannedError,
    traits::frontend::ast::GetType,
    types::{
        EnvRef, Environment,
        builtins::DefineBuiltinsType,
        frontend::{
            SourceFileParsed, Spanned,
            ast::{ASTBlockType, AccessType, AssignType},
            typechecker::{ControlFlow, UVTypeVariable},
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

    pub expected_return_ty: Option<UVType>,
}

impl Typechecker {
    pub fn new(sf: Rc<SourceFileParsed>, name: impl Into<String>) -> Self {
        Self {
            source: sf,
            current_name: name.into(),
            exports: Environment::new(),
            expected_return_ty: None,
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
    ) -> Result<ControlFlow, SpannedError> {
        Ok(match block {
            ASTBlockType::CodeBlock(m) | ASTBlockType::ModuleBlock(m) => {
                self.analyze_group(&m.value, env)?
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

            ASTBlockType::Value(v) => ControlFlow::Simple(v.value.get_type()),
            ASTBlockType::GroupBlock(g) => self.analyze_group(&g.value, env)?,
            ASTBlockType::Return(r) => self.analyze_return(&r.value, env)?,
            ASTBlockType::Continue(_) | ASTBlockType::Break(_) => ControlFlow::Simple(UVType::Void),

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
    ) -> Result<ControlFlow, SpannedError> {
        let new_env = Environment::new_child(env);

        let mut last_type = UVType::Void;
        for node in blocks {
            match self.typecheck(node, new_env.clone())? {
                ControlFlow::Simple(val) => last_type = val,
                cf => return Ok(cf),
            }
        }

        Ok(ControlFlow::Simple(last_type))
    }

    /// Analyze return block
    fn analyze_return(
        &self,
        body: &Option<Box<ASTBlockType>>,
        env: EnvRef<UVTypeVariable>,
    ) -> Result<ControlFlow, SpannedError> {
        let Some(b) = body else {
            return Ok(ControlFlow::Return(UVType::Void));
        };

        match self.typecheck(b, env)? {
            ControlFlow::Simple(val) | ControlFlow::Return(val) => Ok(ControlFlow::Return(val)),
        }
    }
}
