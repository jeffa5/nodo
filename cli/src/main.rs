mod commands;
mod utils;

use anyhow::Result;
use clap::Clap;
use commands::{Opts, SubCommand};
use log::{info, Level};

fn main() -> Result<()> {
    let opts = Opts::parse();

    let log_level = match opts.verbose {
        0 => None,
        1 => Some(Level::Error),
        2 => Some(Level::Warn),
        3 => Some(Level::Info),
        4 => Some(Level::Debug),
        5 => Some(Level::Trace),
        _ => Some(Level::Trace),
    };

    if let Some(l) = log_level {
        simple_logger::init_with_level(l).unwrap();
    }

    info!("raw options: {:?}", opts);

    match opts.subcommand {
        SubCommand::Edit(e) => e.run(&opts.globals),
        SubCommand::Show(s) => s.run(&opts.globals),
    }
}
