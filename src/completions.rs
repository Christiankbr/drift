use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Generator, Shell};

use crate::Cli;

pub fn generate(shell: Shell) -> Result<()> {
    let cmd = Cli::command();

    let mut buf = Vec::new();
    shell.generate(&cmd, &mut buf);

    let output = String::from_utf8(buf)?;
    print!("{}", output);

    Ok(())
}
