use farmfe_core::{
  config::Config, context::CompilationContext, error::Result, parking_lot::Mutex, plugin::Plugin,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct FarmPluginProgress {
  module_count: Arc<Mutex<u32>>,
  progress_bar: ProgressBar,
  first_build: Mutex<bool>,
}

impl FarmPluginProgress {
  pub fn new(_config: &Config) -> Self {
    let spinner_style =
      ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let progress_bar = ProgressBar::new(1);
    progress_bar.set_style(spinner_style.clone());
    progress_bar.set_prefix("[ building ]");

    Self {
      module_count: Arc::new(Mutex::new(0)),
      progress_bar,
      first_build: Mutex::new(true),
    }
  }

  pub fn increment_module_count(&self) {
    let mut count = self.module_count.lock();
    *count += 1;
  }

  pub fn reset_module_count(&self) {
    let mut count = self.module_count.lock();
    *count = 0;
  }

  pub fn get_module_count(&self) -> u32 {
    let count = self.module_count.lock();
    *count
  }
}

impl Plugin for FarmPluginProgress {
  fn name(&self) -> &'static str {
    "FarmPluginProgress"
  }

  fn update_modules(
    &self,
    _params: &mut farmfe_core::plugin::PluginUpdateModulesHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    self.progress_bar.reset();
    self.reset_module_count();
    Ok(None)
  }

  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    self.reset_module_count();
    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    self.increment_module_count();
    let count = self.get_module_count();
    let module = &param.module_id;
    self
      .progress_bar
      .set_message(format!("transform ({count}) {module}"));
    self.progress_bar.inc(1);

    Ok(None)
  }

  fn handle_persistent_cached_module(
    &self,
    module: &farmfe_core::module::Module,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<bool>> {
    self.increment_module_count();
    let count = self.get_module_count();
    let module = &module.id;
    self.progress_bar.set_message(format!(
      "load cached module({count}) {}",
      module.to_string()
    ));
    self.progress_bar.inc(1);

    Ok(None)
  }

  fn module_graph_updated(
    &self,
    _param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParams,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let first_build = self.first_build.lock();

    if !*first_build {
      self.progress_bar.finish_and_clear();
    }

    Ok(None)
  }

  fn render_resource_pot(
    &self,
    param: &farmfe_core::plugin::PluginRenderResourcePotHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>> {
    let first_build = self.first_build.lock();

    if *first_build {
      let name: &String = &param.resource_pot_info.name.clone();
      self.progress_bar.set_message(format!("render {name}"));
      self.progress_bar.inc(1);
    }

    Ok(None)
  }

  fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
    let mut first_build = self.first_build.lock();

    if *first_build {
      self.progress_bar.finish_and_clear();

      *first_build = false;
    }

    Ok(None)
  }
}
