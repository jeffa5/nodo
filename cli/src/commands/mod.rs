use clap::Clap;
use lazy_static::lazy_static;
use std::path::PathBuf;

mod edit;

#[derive(Clap, Debug)]
#[clap(name = "nodo")]
pub struct Opts {
    /// Change the verbosity, repeat for higher levels
    #[clap(short, long, parse(from_occurrences), global = true)]
    pub verbose: u32,

    #[clap(flatten)]
    pub globals: GlobalOpts,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

lazy_static! {
    static ref DATA_DIR: String = dirs::data_dir().unwrap().join("nodo").display().to_string();
}

#[derive(Clap, Debug)]
pub struct GlobalOpts {
    /// The root directory for storing nodos
    #[clap(long, default_value = &DATA_DIR)]
    pub root: PathBuf,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Edit an existing nodo, or create a new one
    Edit(edit::Edit),
}
