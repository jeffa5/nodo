use crate::{commands::GlobalOpts, utils::find_nodo};
use anyhow::{anyhow, Result};
use clap::Clap;
use log::{debug, info};
use nodo_core::{Markdown, Parse, Render};
use std::{
    env, fs,
    fs::File,
    io::{prelude::*, stdin, stdout},
    path::Path,
    process,
};

#[derive(Clap, Debug)]
pub struct Edit {
    /// The target to edit
    #[clap(name = "TARGET")]
    target: String,
}

impl Edit {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let nodo_path = find_nodo(&g.root.join(&self.target));

        if !nodo_path.exists() {
            debug!("Nodo doesn't exist yet");
            self.create_nodo(&nodo_path)?
        }

        let editor = env::var("EDITOR")?;
        info!("executing: '{} {}'", editor, (&nodo_path).display());

        if !process::Command::new(editor)
            .arg(&nodo_path)
            .status()?
            .success()
        {
            return Err(anyhow!("Error occurred when editing. Try running with more verbosity (-v) for more information."));
        }

        // format the just edited nodo
        let mut buf = String::new();
        File::read_to_string(&mut File::open(&nodo_path)?, &mut buf)?;
        let nodo = Markdown::parse(&buf)?;
        Markdown::render(&nodo, &mut File::create(&nodo_path)?)?;

        Ok(())
    }

    fn create_nodo(&self, path: &Path) -> Result<()> {
        print!(
            "Target not found, would you like to create {}? [Y/n]: ",
            self.target
        );
        stdout().lock().flush()?;

        let mut input = String::new();
        stdin().lock().read_line(&mut input)?;

        match input.to_lowercase().trim() {
            "" | "y" | "yes" => {
                println!("Creating {}", path.display());
                if let Some(p) = path.parent() {
                    fs::create_dir_all(p)?;
                }
                File::create(path)?;
            }
            _ => (),
        }

        Ok(())
    }
}
