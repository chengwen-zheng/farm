#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct RemoveConsole {}

impl RemoveConsole {
  fn new(config: &Config, options: String) -> Self {
    Self {}
  }
}

impl Plugin for RemoveConsole {
  fn name(&self) -> &str {
    "RemoveConsole"
  }
}
