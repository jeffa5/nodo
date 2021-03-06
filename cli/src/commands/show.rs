use crate::{
    commands::GlobalOpts,
    utils,
    utils::{target::Target, user},
};
use anyhow::{ensure, Result};
use bat::PrettyPrinter;
use colored::Colorize;
use log::debug;
use nodo_core::{Markdown, Parse};
use std::{cmp::Ordering, fs, fs::File, io::Read, path::Path};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Show {
    /// The target to edit
    #[structopt(name = "TARGET")]
    target: Option<Target>,

    /// Show all, including hidden dirs
    #[structopt(short, long)]
    all: bool,

    /// How many levels to show
    #[structopt(short, long, default_value = "1")]
    depth: i32,

    /// List the available colour themes
    #[structopt(long)]
    list_themes: bool,

    /// Theme to show the nodo with
    #[structopt(long)]
    theme: Option<String>,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            target: None,
            all: false,
            depth: 1,
            list_themes: false,
            theme: None,
        }
    }
}

impl Show {
    pub fn run(&self, g: &GlobalOpts) -> Result<()> {
        if self.list_themes {
            println!("Themes available:");
            for theme in PrettyPrinter::new().themes() {
                println!("{}", theme)
            }
            return Ok(());
        }

        let target = g.root.join(
            self.target
                .as_ref()
                .unwrap_or(&Target::default())
                .build_path(&g.root),
        );

        ensure!(target.exists(), "Target does not exist");

        if target.is_dir() {
            self.print_tree(&target, self.target.is_none(), self.depth)
        } else {
            self.print_nodo(&target)
        }
    }

    fn print_tree(&self, target: &Path, is_root: bool, depth: i32) -> Result<()> {
        debug!("Printing tree from root {}", target.display());
        for entry in read_dir_sorted(target)? {
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

    fn print_nodo(&self, path: &Path) -> Result<()> {
        let mut pp = PrettyPrinter::new();
        pp.input_file(path)
            .language(path.extension().map_or("md", |s| s.to_str().unwrap()));
        if let Some(theme) = self.theme.as_ref() {
            pp.theme(theme);
        }
        pp.print().unwrap();
        Ok(())
    }
}

fn read_dir_sorted(path: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        entries.push(entry?)
    }

    entries.sort_by_key(|e| {
        let path = e.path();
        (path.is_dir(), path)
    });

    Ok(entries)
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
                Ordering::Less => {}
            }
            if files > 0 && directories > 0 {
                print!(", ")
            }
            match directories.cmp(&1) {
                Ordering::Greater => print!("{} directories", directories),
                Ordering::Equal => print!("{} directory", directories),
                Ordering::Less => {}
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
    let children = read_dir_sorted(path)?;
    let children_len = children.len();

    for (i, entry) in children.into_iter().enumerate() {
        let path = entry.path();
        if i == children_len - 1 {
            print!("{}\u{2514}\u{2500} ", prefix);
            if path.is_dir() {
                print_dir_name(&path, depth)?;
                print_dir(&path, &format!("{}   ", prefix), depth - 1)?
            } else {
                print_nodo_summary(&path)?
            }
        } else {
            print!("{}\u{251c}\u{2500} ", prefix);
            if path.is_dir() {
                print_dir_name(&path, depth)?;
                print_dir(&path, &format!("{}\u{2502}  ", prefix), depth - 1)?
            } else {
                print_nodo_summary(&path)?;
            }
        }
    }

    Ok(())
}

fn print_nodo_summary(path: &Path) -> Result<()> {
    let mut buf = String::new();
    File::open(path)?.read_to_string(&mut buf)?;
    let nodo = Markdown::parse(&buf)?;
    let task_count = nodo.count_tasks();
    print!(
        "{}{}",
        user::file_name_string(&path.file_stem().unwrap().to_string_lossy()),
        path.extension()
            .map_or_else(String::new, |e| format!(".{}", e.to_string_lossy()))
    );
    if task_count.total > 0 {
        let task_percentage = format!(
            "{}%",
            (100_f64 * (f64::from(task_count.completed) / f64::from(task_count.total))).trunc()
        );
        print!(
            " [{}/{} ({})]",
            task_count.completed,
            task_count.total,
            if task_count.completed == task_count.total {
                task_percentage.green().bold()
            } else if task_count.completed > task_count.total / 2 {
                task_percentage.yellow().bold()
            } else {
                task_percentage.red().bold()
            }
        )
    }
    println!();
    Ok(())
}
