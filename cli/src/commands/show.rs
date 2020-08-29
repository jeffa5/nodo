use crate::{
    commands::GlobalOpts,
    utils::{target::Target, user},
};
use anyhow::Result;
use clap::Clap;
use log::debug;
use std::{fs, fs::File, path::Path};

#[derive(Clap, Debug, Default)]
pub struct Show {
    /// The target to edit
    #[clap(name = "TARGET")]
    target: Option<Target>,
}

impl Show {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        let target = g.root.join(
            self.target
                .as_ref()
                .unwrap_or(&Target::default())
                .build_path(&g.root),
        );
        if target.exists() {
            if target.is_dir() {
                print_tree(&target)
            } else {
                print_nodo(&target)
            }
        } else {
            Err(anyhow::anyhow!("Target does not exist!"))
        }
    }
}

fn print_tree(root: &Path) -> Result<()> {
    debug!("Printing tree from root {}", root.display());
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            println!(
                "{}",
                user::dir_name_string(&entry.file_name().to_string_lossy())
            );
            print_dir(&path, "")?
        } else {
            print_nodo_summary(&path)?
        }
    }

    Ok(())
}

fn print_dir(path: &Path, prefix: &str) -> Result<()> {
    let children: Vec<_> = fs::read_dir(path)?.collect();
    let children_len = children.len();

    for (i, entry) in children.into_iter().enumerate() {
        let entry = entry?;
        let path = entry.path();
        if i == children_len - 1 {
            print!("{}└─ ", prefix);
            if path.is_dir() {
                println!(
                    "{}",
                    user::dir_name_string(&entry.file_name().to_string_lossy())
                );
                print_dir(&path, &format!("{}   ", prefix))?
            } else {
                print_nodo_summary(&path)?
            }
        } else {
            print!("{}├─ ", prefix);
            if path.is_dir() {
                println!(
                    "{}",
                    user::dir_name_string(&entry.file_name().to_string_lossy())
                );
                print_dir(&path, &format!("{}│  ", prefix))?
            } else {
                print_nodo_summary(&path)?;
            }
        }
    }

    Ok(())
}

fn print_nodo_summary(path: &Path) -> Result<()> {
    println!(
        "{}",
        user::file_name_string(&path.file_name().unwrap().to_string_lossy())
    );
    Ok(())
}

fn print_nodo(path: &Path) -> Result<()> {
    std::io::copy(&mut File::open(path)?, &mut std::io::stdout())?;
    println!();
    Ok(())
}
