use crate::{
    commands::GlobalOpts,
    utils::{target::Target, user},
};
use anyhow::{anyhow, Result};
use clap::Clap;
use log::{debug, info};
use nodo_core::{Markdown, Parse, Render};
use std::{env, fs, fs::File, io::Read, path::Path, process};

#[derive(Clap, Debug)]
pub struct Edit {
    /// The target to edit
    #[clap(name = "TARGET")]
    target: Target,

    /// Create the target if it doesn't exist without a prompt
    ///
    /// This will prevent opening of the editor, designed for scripts
    #[clap(short, long)]
    create: bool,
}

impl Edit {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let nodo_path = &self.target.build_path(&g.root);

        if !nodo_path.exists() {
            debug!("Nodo doesn't exist yet");
            self.create_nodo(nodo_path)?
        }

        // create is designed for scripts so don't open an editor if specified
        if !self.create {
            edit_nodo(nodo_path)?;
        }

        Ok(())
    }

    fn create_nodo(&self, path: &Path) -> Result<()> {
        if self.create || user::confirm("Target not found, would you like to create it?")? {
            if let Some(p) = path.parent() {
                fs::create_dir_all(p)?;
            }
            File::create(path)?;
            println!(
                "Created {}",
                user::file_name_string(path.display().to_string())
            );
        } else {
            return Err(anyhow!("Nodo not created"));
        }

        Ok(())
    }
}

fn edit_nodo(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")?;
    info!("executing: '{} {}'", editor, path.display());

    if !process::Command::new(editor).arg(&path).status()?.success() {
        return Err(anyhow!("Error occurred when editing. Try running with more verbosity (-v) for more information."));
    }

    // format the just edited nodo
    let mut buf = String::new();
    File::read_to_string(&mut File::open(&path)?, &mut buf)?;
    let nodo = Markdown::parse(&buf)?;
    Markdown::render(&nodo, &mut File::create(&path)?)?;

    Ok(())
}
