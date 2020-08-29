use crate::{
    commands::GlobalOpts,
    utils::{target::Target, user},
};
use anyhow::{anyhow, Result};
use clap::Clap;
use colored::*;
use std::fs;

#[derive(Clap, Debug)]
pub struct Remove {
    /// The target to remove
    #[clap(name = "TARGET")]
    target: Target,

    /// Force the removal of the target
    #[clap(short, long)]
    force: bool,
}

impl Remove {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let nodo_path = &self.target.build_path(&g.root);

        if nodo_path.exists() {
            if nodo_path.is_dir() {
                if self.force || user::confirm("This is a directory, are you sure you want to remove it and all of its contents?")?   {
                    fs::remove_dir_all(nodo_path)?;
                    println!("Removed {}", nodo_path.display().to_string().green().bold());
                }
            } else {
                fs::remove_file(nodo_path)?
            }
        }
        // allow a forceful removal to target a non-existant entry
        else if !self.force {
            return Err(anyhow!("Target not found"));
        }

        Ok(())
    }
}
