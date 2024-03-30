use std::collections::HashMap;
use std::sync::Arc;
use farmfe_compiler::Compiler;
use farmfe_core::config::{Config, RuntimeConfig, SourcemapConfig};
use farmfe_core::config::bool_or_obj::BoolOrObj;
use farmfe_core::config::preset_env::PresetEnvConfig;
use farmfe_core::context::CompilationContext;
use farmfe_testing_helpers::fixture;
use farmfe_plugin_remove_console;
use farmfe_plugin_remove_console::RemoveConsole;


#[test]
fn test() {
    fixture!("tests/fixtures/simple/input.js", |file, _cwd| {
            println!("testing: {:?}", file);
            let resolved_path = file.to_string_lossy().to_string();
            let cwd = file.parent().unwrap();
            let config = Config {
              input: HashMap::from([(resolved_path.clone())]),
                   ..Default::default()
            }
            let remove_console_plugin = Arc::new(RemoveConsole::new(
              &config,
              r#"
              {
                "exclude": ["error"]
              }
            "#
              .to_string(),
            ));

            let compiler = Compiler::new(config, vec![remove_console_plugin as _]).unwrap();
            compiler.compile().unwrap();
            let context = compiler.context();
            let resources_map = context.resources_map.lock();

            for (id, resource) in resources_map.iter() {
              let code = std::str::from_utf8(&resource.bytes).unwrap();
              println!("{}: {:?}", id, code);
            }
         });
}


