use clap::Clap;
use lazy_static::lazy_static;
use std::path::PathBuf;

mod archive;
mod edit;
mod r#move;
mod remove;
pub mod show;
mod sync;

#[derive(Clap, Debug)]
#[clap(name = "nodo")]
pub struct Opts {
    /// Change the verbosity, repeat for higher levels
    #[clap(short, long, parse(from_occurrences), global = true)]
    pub verbose: u32,

    #[clap(flatten)]
    pub globals: GlobalOpts,

    #[clap(subcommand)]
    pub subcommand: Option<SubCommand>,
}

lazy_static! {
    static ref DATA_DIR: String = dirs::data_dir().unwrap().join("nodo").display().to_string();
}

#[derive(Clap, Debug)]
pub struct GlobalOpts {
    /// The root directory for storing nodos
    #[clap(long, default_value = &DATA_DIR, env("NODO_ROOT"), global = true)]
    pub root: PathBuf,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Edit an existing nodo, or create a new one
    Edit(edit::Edit),

    /// Show the existing nodos
    Show(show::Show),

    /// Remove the given nodo or directory
    Remove(remove::Remove),

    /// Move a nodo or directory
    Move(r#move::Move),

    /// Archive the given nodo or directory
    Archive(archive::Archive),

    /// Sync the nodo repository
    Sync(sync::Sync),
}
