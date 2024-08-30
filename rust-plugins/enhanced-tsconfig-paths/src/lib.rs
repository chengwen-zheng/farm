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
    if request_path.starts_with('.') || request_path.starts_with("..") {
      return Ok(None);
    }

    if let Some(importer) = &param.importer {
      let importer_dir = PathBuf::from(importer.resolved_path(&context.config.root))
        .parent()
        .unwrap()
        .to_path_buf();
      if self.options.ignore_node_modules && importer_dir.to_str().unwrap().contains("node_modules")
      {
        return Ok(None);
      }

      let mut tsconfig_loader = self.tsconfig_loader.clone();
      let mut matchers = self.matchers.clone();

      if let Some(tsconfig) = tsconfig_loader.load(&importer_dir) {
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
          let extensions = &self.options.tsconfig_paths.extensions;
          for ext in extensions {
            let path_with_ext = format!("{}{}", found_match, ext);
            if std::path::Path::new(&path_with_ext).exists() {
              return Ok(Some(PluginResolveHookResult {
                resolved_path: path_with_ext,
                external: false,
                side_effects: true,
                query: vec![],
                meta: Default::default(),
              }));
            }
          }

          let absolute_path = PathBuf::from(&found_match)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(&found_match));

          let resolved_path = if absolute_path.is_dir() {
            // 检查常见的入口文件名
            let possible_entries = &["index.ts", "index.js", "index.tsx", "index.jsx"];
            let found_entry = possible_entries
              .iter()
              .find(|&entry| absolute_path.join(entry).exists());
            if let Some(entry) = found_entry {
              absolute_path.join(entry)
            } else {
              // 如果没有找到入口文件，使用原始路径
              absolute_path
            }
          } else {
            absolute_path
          };

          // 检查文件是否存在，如果不存在，尝试添加配置的扩展名
          let final_path = if !resolved_path.exists() {
            let file_stem = resolved_path
              .file_stem()
              .unwrap_or_default()
              .to_str()
              .unwrap_or_default();
            let parent = resolved_path.parent().unwrap_or(&resolved_path);

            self
              .options
              .tsconfig_paths
              .extensions
              .iter()
              .find_map(|ext| {
                let path_with_ext = parent.join(format!("{}{}", file_stem, ext));
                if path_with_ext.exists() {
                  Some(path_with_ext)
                } else {
                  None
                }
              })
              .unwrap_or(resolved_path)
          } else {
            resolved_path
          };

          return Ok(Some(PluginResolveHookResult {
            resolved_path: final_path.to_str().unwrap().to_string(),
            external: false,
            side_effects: true,
            query: vec![],
            meta: Default::default(),
          }));
        }
      }
    }

    Ok(None)
  }
}
