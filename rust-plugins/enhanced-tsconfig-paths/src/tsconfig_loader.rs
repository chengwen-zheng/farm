use anyhow::Result;
use farmfe_core::serde_json;
use farmfe_core::serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct TsconfigLoader {
  visited_dirs: HashMap<PathBuf, bool>,
}

#[derive(Clone, Debug)]
pub struct TsconfigResult {
  pub config_file_abs_path: String,
  pub absolute_base_url: String,
  pub paths: HashMap<String, Vec<String>>,
}

impl TsconfigLoader {
  pub fn new() -> Self {
    TsconfigLoader {
      visited_dirs: HashMap::new(),
    }
  }

  pub fn load(&mut self, cwd: &Path) -> Option<TsconfigResult> {
    let tsconfig_dir = self.get_closest_tsconfig_dir(cwd)?;
    let tsconfig_path = tsconfig_dir.join("tsconfig.json");

    let tsconfig_content = std::fs::read_to_string(&tsconfig_path).ok()?;
    let tsconfig: Value = serde_json::from_str(&tsconfig_content).ok()?;

    let base_url = tsconfig.get("compilerOptions")?.get("baseUrl")?.as_str()?;
    let paths = tsconfig.get("compilerOptions")?.get("paths")?;

    let absolute_base_url = tsconfig_dir.join(base_url).to_str()?.to_string();
    let paths_map: HashMap<String, Vec<String>> = paths
      .as_object()?
      .iter()
      .map(|(k, v)| {
        (
          k.clone(),
          v.as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect(),
        )
      })
      .collect();

    Some(TsconfigResult {
      config_file_abs_path: tsconfig_path.to_str()?.to_string(),
      absolute_base_url,
      paths: paths_map,
    })
  }

  fn get_closest_tsconfig_dir(&mut self, cwd: &Path) -> Option<PathBuf> {
    if cwd.parent() == Some(cwd) {
      return None;
    }

    if let Some(&found) = self.visited_dirs.get(cwd) {
      return if found { Some(cwd.to_path_buf()) } else { None };
    }

    let tsconfig_path = cwd.join("tsconfig.json");
    if tsconfig_path.exists() {
      self.visited_dirs.insert(cwd.to_path_buf(), true);
      Some(cwd.to_path_buf())
    } else {
      self.visited_dirs.insert(cwd.to_path_buf(), false);
      self.get_closest_tsconfig_dir(cwd.parent()?)
    }
  }
}
