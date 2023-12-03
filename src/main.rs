#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

pub(crate) mod app;
pub(crate) mod example_id;
pub(crate) mod repl;

use clap::Parser;
use futures::{FutureExt, StreamExt};
use itertools::Itertools;

use crate::{
    app::{Inputs, Outputs},
    repl::driver::ReplDriver,
};

#[derive(Debug, clap::Parser)]
#[command()]
struct Cli {
    /// Path to a `nix` executable
    nix_path: camino::Utf8PathBuf,
    /// pattern (`glob` crate) of markdown filespaths
    sources: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let repl_examples = repl::example::obtain(&cli.sources)?;
    if repl_examples.is_empty() {
        anyhow::bail!("could not find any REPL examples");
    }
    let (repl_driver, repl_events) = ReplDriver::new(cli.nix_path);

    let inputs = Inputs {
        repl_examples,
        repl_events: repl_events.boxed_local(),
    };

    let outputs = app::app(inputs);

    let Outputs {
        repl_commands,
        done,
        execution_handle,
        eprintln_strings,
    } = outputs;

    let eprintln_task = eprintln_strings.for_each(|string| async move {
        eprintln!("{string}");
    });
    let repl_task = repl_driver.init(repl_commands);

    tokio::select! {
        _ = execution_handle.fuse() => unreachable!(),
        _ = eprintln_task.fuse() => unreachable!(),
        _ = repl_task.fuse() => unreachable!(),
        done = done.fuse() => done,
    }
}

#[derive(Debug, Clone, derive_more::Display)]
#[display("{}", _0)]
struct Eprintln(String);

#[derive(Debug, derive_more::Deref)]
struct PtyLine(String);

impl std::str::FromStr for PtyLine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.chars()
            .with_position()
            .try_for_each(|(position, character)| {
                use itertools::Position::*;
                match (position, character) {
                    (Last | Only, '\n') => Ok(()),
                    (Last | Only, _) => Err(anyhow::anyhow!("does not end with LF {s:?}")),
                    (_, '\n') => Err(anyhow::anyhow!("LF before end {s:?}")),
                    _ => Ok(()),
                }
            })?;

        Ok(Self(s.to_string()))
    }
}
