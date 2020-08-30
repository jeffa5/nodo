use crate::{
    commands::GlobalOpts,
    utils,
    utils::{target::Target, user},
};
use anyhow::Result;
use clap::Clap;
use log::debug;
use std::{cmp::Ordering, fs, fs::File, path::Path};

#[derive(Clap, Debug)]
pub struct Show {
    /// The target to edit
    #[clap(name = "TARGET")]
    target: Option<Target>,

    /// Show all, including hidden dirs
    #[clap(short, long)]
    all: bool,

    /// How many levels to show
    #[clap(short, long, default_value = "1")]
    depth: i32,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            target: None,
            all: false,
            depth: 1,
        }
    }
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
                self.print_tree(&target, self.target.is_none(), self.depth)
            } else {
                print_nodo(&target)
            }
        } else {
            Err(anyhow::anyhow!("Target does not exist!"))
        }
    }

    fn print_tree(&self, target: &Path, is_root: bool, depth: i32) -> Result<()> {
        debug!("Printing tree from root {}", target.display());
        for entry in fs::read_dir(target)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy() == ".git" {
                continue;
            }
            if !self.all && is_root && utils::is_hidden_dir(&entry.file_name().to_string_lossy()) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                print_dir_name(&path, depth)?;
                print_dir(&path, "", depth - 1)?
            } else {
                print_nodo_summary(&path)?
            }
        }

        Ok(())
    }
}

fn print_dir_name(path: &Path, depth: i32) -> Result<()> {
    print!(
        "{}",
        user::dir_name_string(&path.file_name().unwrap().to_string_lossy())
    );
    if depth == 1 {
        let (files, directories) = fs::read_dir(&path)?.fold((0, 0), |(f, d), e| {
            if e.unwrap().path().is_dir() {
                (f, d + 1)
            } else {
                (f + 1, d)
            }
        });
        if files > 0 || directories > 0 {
            print!(" [");
            match files.cmp(&1) {
                Ordering::Greater => print!("{} files", files),
                Ordering::Equal => print!("{} file", files),
                _ => (),
            }
            if files > 0 && directories > 0 {
                print!(", ")
            }
            match directories.cmp(&1) {
                Ordering::Greater => print!("{} directories", directories),
                Ordering::Equal => print!("{} directory", directories),
                _ => (),
            }

            print!("]");
        }
    }
    println!();
    Ok(())
}

fn print_dir(path: &Path, prefix: &str, depth: i32) -> Result<()> {
    if depth == 0 {
        return Ok(());
    }
    let children: Vec<_> = fs::read_dir(path)?.collect();
    let children_len = children.len();

    for (i, entry) in children.into_iter().enumerate() {
        let entry = entry?;
        let path = entry.path();
        if i == children_len - 1 {
            print!("{}└─ ", prefix);
            if path.is_dir() {
                print_dir_name(&path, depth)?;
                print_dir(&path, &format!("{}   ", prefix), depth - 1)?
            } else {
                print_nodo_summary(&path)?
            }
        } else {
            print!("{}├─ ", prefix);
            if path.is_dir() {
                print_dir_name(&path, depth)?;
                print_dir(&path, &format!("{}│  ", prefix), depth - 1)?
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
