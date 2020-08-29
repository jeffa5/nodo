use crate::{
    commands::GlobalOpts,
    utils::{target::Target, user},
};
use anyhow::{anyhow, Result};
use clap::Clap;
use std::fs;

#[derive(Clap, Debug)]
pub struct Move {
    /// The source to move
    #[clap(name = "SOURCE")]
    source: Target,

    /// The destination to move the source to
    #[clap(name = "DESTINATION")]
    destination: Target,
}

impl Move {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let source_path = &self.source.build_path(&g.root);
        let destination_path = &self.destination.build_path(&g.root);

        if source_path.exists() {
            let mut destination_path = g.root.join(destination_path);
            if destination_path.exists() && !destination_path.is_dir() {
                return Err(anyhow!("Destination already exists and is a file"));
            }

            if source_path.is_dir() {
                if destination_path.is_dir() {
                    let dest = destination_path.join(source_path.file_name().unwrap());
                    fs::rename(source_path, &dest)?;
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
                    println!(
                        "Moved dir {} to {}",
                        user::dir_name_string(source_path.display().to_string()),
                        user::dir_name_string(destination_path.display().to_string())
                    )
                }
            } else if destination_path.is_dir() {
                let dest = destination_path.join(source_path.file_name().unwrap());
                fs::rename(source_path, &dest)?;
                println!(
                    "Moved file {} to {}",
                    user::file_name_string(source_path.display().to_string()),
                    user::file_name_string(dest.display().to_string())
                )
            } else {
                fs::create_dir_all(destination_path.parent().unwrap())?;
                fs::rename(source_path, &destination_path)?;
                println!(
                    "Moved file {} to {}",
                    user::file_name_string(source_path.display().to_string()),
                    user::file_name_string(destination_path.display().to_string())
                )
            }
        } else {
            return Err(anyhow!("Source not found"));
        }

        Ok(())
    }
}
