use log::debug;
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
        debug!("Built raw full path: {:?}", full_path);

        if !full_path.exists() {
            let path = with_md_extension(&full_path);
            debug!(
                "Built full path with extension since argument didn't exist: {:?}",
                path
            );
            path
        } else {
            debug!(
                "Built full path without adding extension since argument already exists: {:?}",
                full_path
            );
            full_path
        }
    }
}

fn with_md_extension(path: &Path) -> PathBuf {
    match path.extension() {
        None => path.with_extension("md"),
        Some(_) => path.to_owned(),
    }
}
