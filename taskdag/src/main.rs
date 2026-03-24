mod db;
mod task;
mod cli;

use anyhow::Result;
use clap::Parser;
use cli::TaskDagCLI;

fn main() -> Result<()> {
    let cli = TaskDagCLI::parse();
    cli.execute()
}