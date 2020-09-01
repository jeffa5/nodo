use crate::{
    commands::GlobalOpts,
    utils::{git, target::Target, user},
};
use anyhow::{ensure, Result};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Move {
    /// The source to move
    #[structopt(name = "SOURCE")]
    source: Target,

    /// The destination to move the source to
    #[structopt(name = "DESTINATION")]
    destination: Target,
}

impl Move {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let source_path = &self.source.build_path(&g.root);
        let destination_path = &self.destination.build_path(&g.root);

        ensure!(source_path.exists(), "Source not found");

        let mut destination_path = g.root.join(destination_path);

        ensure!(
            !destination_path.exists() || destination_path.is_dir(),
            "Destination already exists and is a file"
        );

        let mut repo = git::Repo::open(&g.root)?;

        if source_path.is_dir() {
            if destination_path.is_dir() {
                let dest = destination_path.join(source_path.file_name().unwrap());
                fs::rename(source_path, &dest)?;
                repo.add_path(source_path)?.add_path(&dest)?;
                println!(
                    "Moved dir {} to {}",
                    user::dir_name_string(source_path.display().to_string()),
                    user::dir_name_string(dest.display().to_string())
                )
            }
            // doesn't exist
            else {
                fs::create_dir_all(destination_path.parent().unwrap())?;
                destination_path.set_extension("");
                fs::rename(source_path, &destination_path)?;
                repo.add_path(source_path)?.add_path(&destination_path)?;
                println!(
                    "Moved dir {} to {}",
                    user::dir_name_string(source_path.display().to_string()),
                    user::dir_name_string(destination_path.display().to_string())
                )
            }
        } else if destination_path.is_dir() {
            let dest = destination_path.join(source_path.file_name().unwrap());
            fs::rename(source_path, &dest)?;
            repo.add_path(source_path)?.add_path(&dest)?;
            println!(
                "Moved file {} to {}",
                user::file_name_string(source_path.display().to_string()),
                user::file_name_string(dest.display().to_string())
            )
        } else {
            fs::create_dir_all(destination_path.parent().unwrap())?;
            fs::rename(source_path, &destination_path)?;
            repo.add_path(source_path)?.add_path(&destination_path)?;
            println!(
                "Moved file {} to {}",
                user::file_name_string(source_path.display().to_string()),
                user::file_name_string(destination_path.display().to_string())
            )
        }

        Ok(())
    }
}
