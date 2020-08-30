#![feature(never_type)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]

mod commands;
mod utils;

use anyhow::Result;
use clap::Clap;
use commands::{show::Show, Opts, SubCommand};
use log::{info, Level};

fn main() -> Result<()> {
    let opts = Opts::parse();

    let log_level = match opts.verbose {
        0 => None,
        1 => Some(Level::Error),
        2 => Some(Level::Warn),
        3 => Some(Level::Info),
        4 => Some(Level::Debug),
        _ => Some(Level::Trace),
    };

    if let Some(l) = log_level {
        simple_logger::init_with_level(l).unwrap();
    }

    info!("raw options: {:?}", opts);

    match opts.subcommand {
        None => Show::default().run(&opts.globals),
        Some(cmd) => match cmd {
            SubCommand::Edit(e) => e.run(&opts.globals),
            SubCommand::Show(s) => s.run(&opts.globals),
            SubCommand::Remove(r) => r.run(&opts.globals),
            SubCommand::Move(m) => m.run(&opts.globals),
            SubCommand::Archive(a) => a.run(&opts.globals),
            SubCommand::Sync(s) => s.run(&opts.globals),
        },
    }
}
