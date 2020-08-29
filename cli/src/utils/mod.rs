use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug)]
pub struct Target(String);

impl FromStr for Target {
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Target(s.to_string()))
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Target {
    fn default() -> Self {
        Self(String::default())
    }
}

impl Target {
    pub fn build_path(&self, root: &Path) -> PathBuf {
        let full_path = root.join(&self.0);

        if !full_path.exists() {
            with_md_extension(&full_path)
        } else if full_path.is_dir() {
            full_path
        } else {
            with_md_extension(&full_path)
        }
    }
}

fn with_md_extension(path: &Path) -> PathBuf {
    match path.extension() {
        None => path.with_extension("md"),
        Some(_) => path.to_owned(),
    }
}
