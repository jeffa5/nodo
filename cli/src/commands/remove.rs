use crate::{
    commands::GlobalOpts,
    utils::{target::Target, user},
};
use anyhow::{bail, Result};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Remove {
    /// The target to remove
    #[structopt(name = "TARGET")]
    target: Target,

    /// Force the removal of the target
    #[structopt(short, long)]
    force: bool,
}

impl Remove {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let nodo_path = &self.target.build_path(&g.root);

        if nodo_path.exists() {
            if nodo_path.is_dir() {
                if self.force || user::confirm("This is a directory, are you sure you want to remove it and all of its contents?")?   {
                    fs::remove_dir_all(nodo_path)?;
                    println!("Removed {}", user::dir_name_string(nodo_path.display().to_string()));
                }
            } else {
                fs::remove_file(nodo_path)?;
                println!(
                    "Removed {}",
                    user::file_name_string(nodo_path.display().to_string())
                );
            }
        }
        // allow a forceful removal to target a non-existant entry
        else if !self.force {
            bail!("Target not found");
        }

        Ok(())
    }
}
