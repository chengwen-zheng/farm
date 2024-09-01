use glob::Pattern;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct PathMatcher {
  base_url: String,
  paths: HashMap<String, Vec<Pattern>>,
  main_fields: Vec<String>,
  match_all: bool,
}

impl PathMatcher {
  pub fn new(
    base_url: &str,
    paths: &HashMap<String, Vec<String>>,
    main_fields: &[String],
    match_all: bool,
  ) -> Self {
    let compiled_paths: HashMap<String, Vec<Pattern>> = paths
      .iter()
      .map(|(k, v)| {
        (
          k.clone(),
          v.iter().map(|p| Pattern::new(p).unwrap()).collect(),
        )
      })
      .collect();

    PathMatcher {
      base_url: base_url.to_string(),
      paths: compiled_paths,
      main_fields: main_fields.to_vec(),
      match_all,
    }
  }

  pub fn match_path(&self, request_path: &str, extensions: &[String]) -> Option<String> {
    println!("Matching path: {}", request_path);
    for (pattern, substitutions) in &self.paths {
      let pattern = if self.match_all {
        format!("{}*", pattern)
      } else {
        pattern.clone()
      };
      if Pattern::new(&pattern).unwrap().matches(request_path) {
        for substitution in substitutions {
          let potential_path = Path::new(&self.base_url)
            .join(substitution.as_str().replace("*", request_path))
            .to_str()
            .unwrap()
            .to_string();

          if Path::new(&potential_path).exists() {
            return Some(potential_path);
          }

          for ext in extensions {
            let with_extension: String = format!("{}{}", potential_path, ext);
            if Path::new(&with_extension).exists() {
              return Some(with_extension);
            }
          }
        }
      }
    }

    None
  }
}
