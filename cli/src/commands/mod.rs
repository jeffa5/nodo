use lazy_static::lazy_static;
use std::path::PathBuf;
use structopt::StructOpt;

mod completions;
mod edit;
mod r#move;
mod remove;
pub mod show;
mod sync;

#[derive(StructOpt, Debug)]
#[structopt(name = "nodo")]
pub struct Opts {
    /// Change the verbosity, repeat for higher levels
    #[structopt(short, long, parse(from_occurrences), global = true)]
    pub verbose: u32,

    #[structopt(flatten)]
    pub globals: GlobalOpts,

    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

lazy_static! {
    static ref DATA_DIR: String = dirs::data_dir().unwrap().join("nodo").display().to_string();
}

#[derive(StructOpt, Debug)]
pub struct GlobalOpts {
    /// The root directory for storing nodos
    #[structopt(long, default_value = &DATA_DIR, env("NODO_ROOT"), global = true)]
    pub root: PathBuf,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// Edit an existing nodo, or create a new one
    Edit(edit::Edit),

    /// Show the existing nodos
    Show(show::Show),

    /// Remove the given nodo or directory
    Remove(remove::Remove),

    /// Move a nodo or directory
    Move(r#move::Move),

    /// Sync the nodo repository
    Sync(sync::Sync),

    /// Generate completions for the given shell
    Completions(completions::Completions),
}
