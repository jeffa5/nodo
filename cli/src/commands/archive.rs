use crate::{
    commands::GlobalOpts,
    utils::{target::Target, user},
};
use anyhow::{anyhow, Result};
use clap::Clap;
use std::{fs, path::Path};

#[derive(Clap, Debug)]
pub struct Archive {
    /// The target to archive
    #[clap(name = "TARGET")]
    target: Target,
}

impl Archive {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let source_path = self.target.build_path(&g.root);

        if source_path.exists() {
            let source_is_dir = source_path.is_dir();
            let archived_path = g
                .root
                .join(Path::new("archive").join(&source_path.strip_prefix(&g.root)?));

            fs::create_dir_all(archived_path.parent().unwrap())?;
            fs::rename(&source_path, archived_path)?;

            if source_is_dir {
                println!(
                    "Archived {}",
                    user::dir_name_string(source_path.display().to_string())
                )
            } else {
                println!(
                    "Archived {}",
                    user::file_name_string(source_path.display().to_string())
                )
            }
        } else {
            return Err(anyhow!("Target not found"));
        }

        Ok(())
    }
}
