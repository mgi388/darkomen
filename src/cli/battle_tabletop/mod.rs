use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::battle_tabletop::*;

#[derive(Args)]
pub struct BattleTabletopArgs {
    #[command(subcommand)]
    pub subcommand: Option<BattleTabletopSubcommands>,
}

#[derive(Subcommand)]
pub enum BattleTabletopSubcommands {
    Edit(EditBattleTabletopArgs),
}

#[derive(Args)]
pub struct EditBattleTabletopArgs {
    /// The path to the battle tabletop file to edit, e.g., ".../B1_01.BTB".
    #[arg(index = 1)]
    pub battle_tabletop_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the battle tabletop file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Clone, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

pub fn run(args: &BattleTabletopArgs) -> anyhow::Result<()> {
    if let Some(BattleTabletopSubcommands::Edit(edit_args)) = &args.subcommand {
        edit_battle_tabletop_file(edit_args)?;
    }

    Ok(())
}

fn edit_battle_tabletop_file(args: &EditBattleTabletopArgs) -> anyhow::Result<()> {
    let battle_tabletop_file: PathBuf = args.battle_tabletop_file.clone().into();

    // Load the battle tabletop file.
    let file = File::open(battle_tabletop_file.clone())?;
    let battle_tabletop = Decoder::new(file).decode()?;

    // Serialize the battle tabletop to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&battle_tabletop, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&battle_tabletop)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        battle_tabletop_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("battle_tabletop"),
    );
    let suffix = format!(".{extension}");
    let mut temp_file = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(&suffix)
        .tempfile()?;
    temp_file.write_all(as_string.as_bytes())?;
    temp_file.flush()?;

    // Open the temporary file in the editor.
    let (editor, editor_args) = {
        let mut parts = args.editor.split_whitespace();
        let editor = parts.next().unwrap();
        let editor_args = parts.collect::<Vec<_>>();
        (editor, editor_args)
    };
    let mut command = std::process::Command::new(editor);
    command.args(editor_args);

    // This call blocks until the editor process exits.
    println!("Waiting for editor to close...");
    command.arg(temp_file.path()).status()?;
    println!("Editor closed");

    // Read the modified human-readable string from the temporary file.
    let mut modified_string = String::new();
    temp_file.reopen()?.read_to_string(&mut modified_string)?;

    // Deserialize the modified string to an battle tabletop.
    let modified_battle_tabletop = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified battle tabletop to the original file.
    let file = File::create(battle_tabletop_file)?;
    Encoder::new(file).encode(&modified_battle_tabletop)?;

    println!("Battle tabletop file successfully edited");

    Ok(())
}
