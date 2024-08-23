use farmfe_core::serde_json;
use farmfe_core::{config::Config, plugin::Plugin};
use std::collections::HashMap;
use std::path::PathBuf;

use farmfe_macro_plugin::farm_plugin;
mod path_matcher;
mod tsconfig_loader;
use farmfe_core::plugin::PluginResolveHookResult;
use path_matcher::PathMatcher;

use tsconfig_loader::TsconfigLoader;

#[serde(rename_all = "camelCase", default)]
#[derive(Clone, Debug, serde::Deserialize, Default)]
pub struct Options {
  ignore_node_modules: bool,
  tsconfig_paths: TsconfigPathsOptions,
}

#[serde(rename_all = "camelCase", default)]
#[derive(Clone, Debug, serde::Deserialize, Default)]
pub struct TsconfigPathsOptions {
  main_fields: Vec<String>,
  extensions: Vec<String>,
  match_all: bool,
}

#[farm_plugin]
pub struct EnhancedTsconfigPaths {
  root: PathBuf,
  options: Options,
  tsconfig_loader: TsconfigLoader,
  matchers: HashMap<String, PathMatcher>,
}

impl EnhancedTsconfigPaths {
  fn new(config: &Config, options: String) -> Self {
    let plugin_options = serde_json::from_str::<Options>(&options).unwrap();
    Self {
      root: PathBuf::from(&config.root),
      options: plugin_options,
      tsconfig_loader: TsconfigLoader::new(),
      matchers: HashMap::new(),
    }
  }
}

impl Plugin for EnhancedTsconfigPaths {
  fn name(&self) -> &str {
    "EnhancedTsconfigPaths"
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    let request_path = &param.source;
    println!("request_path: {:?}", request_path);
    if request_path.starts_with('.') || request_path.starts_with("..") {
      return Ok(None);
    }

    if let Some(importer) = &param.importer {
      let importer_dir = PathBuf::from(importer.resolved_path(&context.config.root))
        .parent()
        .unwrap()
        .to_path_buf();
      println!("importer_dir: {:?}", importer_dir);
      if self.options.ignore_node_modules && importer_dir.to_str().unwrap().contains("node_modules")
      {
        return Ok(None);
      }

      let mut tsconfig_loader = self.tsconfig_loader.clone();
      let mut matchers = self.matchers.clone();

      if let Some(tsconfig) = tsconfig_loader.load(&importer_dir) {
        println!("tsconfig: {:?}", tsconfig);
        let matcher = matchers
          .entry(tsconfig.config_file_abs_path.clone())
          .or_insert_with(|| {
            PathMatcher::new(
              &tsconfig.absolute_base_url,
              &tsconfig.paths,
              &self.options.tsconfig_paths.main_fields,
              self.options.tsconfig_paths.match_all,
            )
          });

        if let Some(found_match) =
          matcher.match_path(request_path, &self.options.tsconfig_paths.extensions)
        {
          println!("found_match: {:?}", found_match);
          let extensions = &self.options.tsconfig_paths.extensions;
          for ext in extensions {
            let path_with_ext = format!("{}{}", found_match, ext);
            if std::path::Path::new(&path_with_ext).exists() {
              println!(
                "Returning resolved path with extension: {:?}",
                path_with_ext
              );
              return Ok(Some(PluginResolveHookResult {
                resolved_path: path_with_ext,
                external: false,
                side_effects: true,
                query: vec![],
                meta: Default::default(),
              }));
            }
          }

          println!("Returning original resolved path: {:?}", found_match);


          let absolute_path = PathBuf::from(&found_match)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(&found_match))
            .to_str()
            .unwrap()
            .to_string();
          return Ok(Some(PluginResolveHookResult {
            resolved_path: absolute_path,
            external: false,
            side_effects: true,
            query: vec![],
            meta: Default::default(),
          }));
        }
      }
    }

    // self.tsconfig_loader = tsconfig_loader;
    // self.matchers = matchers;

    Ok(None)
  }
}
