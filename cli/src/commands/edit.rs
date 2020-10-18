use crate::{
    commands::GlobalOpts,
    utils::{git, target::Target, user},
};
use anyhow::{ensure, Result};
use log::{debug, info};
use nodo_core::{Markdown, Parse, Render};
use std::{env, fs, fs::File, io::Read, path::Path, process};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Edit {
    /// The target to edit
    #[structopt(name = "TARGET")]
    target: Target,

    /// Create the target if it doesn't exist without a prompt
    #[structopt(short, long)]
    create: bool,
}

impl Edit {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let nodo_path = &self.target.build_path(&g.root);

        if !nodo_path.exists() {
            debug!("Nodo doesn't exist yet");
            self.create_nodo(nodo_path)?
        }

        ensure!(nodo_path.is_file(), "Nodo to edit must be a file");

        edit_nodo(nodo_path, &g.root)?;

        Ok(())
    }

    fn create_nodo(&self, path: &Path) -> Result<()> {
        ensure!(
            self.create
                || user::confirm(&format!(
                    "{} not found, would you like to create it?",
                    user::file_name_string(self.target.to_string())
                ))?,
            "Nodo not created"
        );
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
        File::create(path)?;
        println!(
            "Created {}",
            user::file_name_string(path.display().to_string())
        );

        Ok(())
    }
}

fn edit_nodo(path: &Path, root: &Path) -> Result<()> {
    let editor = env::var("EDITOR")?;
    info!("executing: '{} {}'", editor, path.display());

    ensure!(
        process::Command::new(editor).arg(&path).status()?.success(),
        "Error occurred when editing. Try running with more verbosity (-v) for more information."
    );

    // format the just edited nodo
    let mut buf = String::new();
    File::read_to_string(&mut File::open(&path)?, &mut buf)?;
    let nodo = Markdown::parse(&buf)?;
    Markdown::render(&nodo, &mut File::create(&path)?)?;

    commit_changes(path, root)?;

    Ok(())
}

fn commit_changes(path: &Path, root: &Path) -> Result<()> {
    git::Repo::open(root)?.add_path(path)?.commit()
}
