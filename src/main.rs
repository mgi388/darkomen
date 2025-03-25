mod cli;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "darkomen", version, about, next_line_help(false))]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
    Army(cli::army::ArmyArgs),
    BattleTabletop(cli::battle_tabletop::BattleTabletopArgs),
    Gameflow(cli::gameflow::GameflowArgs),
    Project(cli::project::ProjectArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        Subcommands::Army(args) => cli::army::run(&args)?,
        Subcommands::BattleTabletop(args) => cli::battle_tabletop::run(&args)?,
        Subcommands::Gameflow(args) => cli::gameflow::run(&args)?,
        Subcommands::Project(args) => cli::project::run(&args)?,
    }

    Ok(())
}
