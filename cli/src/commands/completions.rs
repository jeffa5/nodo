use crate::commands::Opts;
use anyhow::Result;
use clap::Shell;
use std::io;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Completions {
    /// The shell to generate completions for, one of: zsh, bash, fish, powershell, elvish
    #[structopt(name = "SHELL")]
    shell: Shell,
}

impl Completions {
    pub fn run(&self) -> Result<()> {
        Opts::clap().gen_completions_to("nodo", self.shell, &mut io::stdout());
        Ok(())
    }
}
