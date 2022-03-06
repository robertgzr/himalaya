//! TUI args module.
//!
//! This module provides subcommands related to the TUI.

use anyhow::Result;
use clap::{self, App, SubCommand};
use log::info;

const START_SUBCMD: &str = "tui";

/// Represents the TUI commands.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    /// Represents the start TUI command.
    Start,
}

/// Represents the TUI command matcher.
pub fn matches(m: &clap::ArgMatches) -> Result<Option<Cmd>> {
    info!(">> tui command matcher");

    let cmd = if let Some(_m) = m.subcommand_matches(START_SUBCMD) {
        info!("start tui command matched");
        Some(Cmd::Start)
    } else {
        None
    };

    info!("<< tui command matcher");
    Ok(cmd)
}

/// Represents the TUI subcommands.
pub fn subcmds<'a>() -> Vec<App<'a, 'a>> {
    vec![SubCommand::with_name(START_SUBCMD).about("Starts the TUI")]
}
