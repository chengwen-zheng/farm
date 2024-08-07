#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct EnhancedTsconfigPaths {}

impl EnhancedTsconfigPaths {
  fn new(_config: &Config, _options: String) -> Self {
    Self {}
  }
}

impl Plugin for EnhancedTsconfigPaths {
  fn name(&self) -> &str {
    "EnhancedTsconfigPaths"
  }

  fn resolve(
      &self,
      param: &farmfe_core::plugin::PluginResolveHookParam,
      _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
      _hook_context: &farmfe_core::plugin::PluginHookContext,
    ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
      println!("EnhancedTsconfigPaths resolve");
      println!("{:?}", param);
      todo!()
  }
}
