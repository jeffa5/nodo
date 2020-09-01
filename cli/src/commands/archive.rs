use crate::{
    commands::GlobalOpts,
    utils::{git, target::Target, user},
};
use anyhow::{ensure, Result};
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Archive {
    /// The target to archive
    #[structopt(name = "TARGET")]
    target: Target,
}

impl Archive {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let source_path = self.target.build_path(&g.root);

        ensure!(source_path.exists(), "Target not found");

        let source_is_dir = source_path.is_dir();
        let archived_path = g
            .root
            .join(Path::new("archive").join(&source_path.strip_prefix(&g.root)?));

        let mut repo = git::Repo::open(&g.root)?;

        fs::create_dir_all(archived_path.parent().unwrap())?;
        fs::rename(&source_path, &archived_path)?;
        repo.add_path(&source_path)?.add_path(&archived_path)?;

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

        Ok(())
    }
}
