#![feature(rustc_private)]
#![deny(clippy::all)]
use farmfe_core::{config::Config, plugin::Plugin, serde_json};
use farmfe_core::serde::Deserialize;
use farmfe_core::swc_common::Mark;
use farmfe_core::swc_ecma_ast::{Callee, CallExpr, Expr, Ident, MemberProp, Program, Stmt};
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::swc_ecma_utils::swc_common::{DUMMY_SP, SyntaxContext};
use farmfe_toolkit::swc_ecma_utils::swc_ecma_ast::EmptyStmt;
use farmfe_toolkit::swc_ecma_visit::{Fold, FoldWith, noop_fold_type};

#[farm_plugin]
pub struct RemoveConsole {
    exclude: Vec<String>,
    unresolved_ctxt: SyntaxContext,
}


#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum RemoveConsoleConfig {
    All(bool),
    WithOptions(Options),
}

impl RemoveConsoleConfig {
    pub fn truthy(&self) -> bool {
        match self {
            RemoveConsoleConfig::All(b) => *b,
            RemoveConsoleConfig::WithOptions(_) => true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Options {
    #[serde(default)]
    pub exclude: Vec<String>,
}


impl RemoveConsole {
    pub fn new(config: &Config, options: String) -> Self {
        let remove_console_config: RemoveConsoleConfig = serde_json::from_str::<RemoveConsoleConfig>(&options)
            .expect("invalid options")
            .unwrap_or_else(|| RemoveConsoleConfig::All(true));
        let exclude = match remove_console_config {
            Config::WithOptions(x) => x.exclude,
            _ => vec![],
        };

        Self {
            exclude,
            unresolved_ctxt: DUMMY_SP,
        }
    }

    fn remove_console(unresolved_ctxt: SyntaxContext) -> impl Fold {
        RemoveConsole { exclude: vec![], unresolved_ctxt }
    }
}

impl Plugin for RemoveConsole {
    fn name(&self) -> &str {
        "RemoveConsole"
    }

    fn transform(
        &self,
        param: &farmfe_core::plugin::PluginTransformHookParam,
        context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
        let ast = param.meta.as_script_mut().take_ast();
        let mut program = Program::Module(ast);
        let unresolved_mark = Mark::from_u32(param.meta.as_script_mut().unresolved_mark);
        program.fold_with(Self::remove_console(SyntaxContext::empty().apply_mark(unresolved_mark)));
        Ok(None)
    }
}

impl RemoveConsole {
    fn is_global_console(&self, ident: &Ident) -> bool {
        &ident.sym == "console" && ident.span.ctxt == self.unresolved_ctxt
    }

    fn should_remove_call(&mut self, n: &CallExpr) -> bool {
        let callee = &n.callee;
        let member_expr = match callee {
            Callee::Expr(e) => match &**e {
                Expr::Member(m) => m,
                _ => return false,
            },
            _ => return false,
        };

        // Don't attempt to evaluate computed properties.
        if matches!(&member_expr.prop, MemberProp::Computed(..)) {
            return false;
        }

        // Only proceed if the object is the global `console` object.
        match &*member_expr.obj {
            Expr::Ident(i) if self.is_global_console(i) => {}
            _ => return false,
        }

        // Check if the property is requested to be excluded.
        // Here we do an O(n) search on the list of excluded properties because the size
        // should be small.
        match &member_expr.prop {
            MemberProp::Ident(i) if !self.exclude.iter().any(|x| *x == i.sym) => {}
            _ => return false,
        }

        true
    }
}

impl Fold for RemoveConsole {
    noop_fold_type!();
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt {
        if let Stmt::Expr(e) = &stmt {
            if let Expr::Call(c) = &*e.expr {
                if self.should_remove_call(c) {
                    return Stmt::Empty(EmptyStmt { span: DUMMY_SP });
                }
            }
        }
        stmt.fold_children_with(self)
    }
}