use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::army::*;

#[derive(Debug, Args)]
pub struct ArmyArgs {
    #[command(subcommand)]
    pub subcommand: Option<ArmySubcommands>,
}

#[derive(Debug, Subcommand)]
pub enum ArmySubcommands {
    Edit(EditArmyArgs),
}

#[derive(Debug, Args)]
pub struct EditArmyArgs {
    /// The path to the army file to edit, e.g., ".../B1_01/B101MRC.ARM".
    #[arg(index = 1)]
    pub army_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the army file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

pub fn run(args: &ArmyArgs) -> anyhow::Result<()> {
    if let Some(ArmySubcommands::Edit(edit_args)) = &args.subcommand {
        edit_army_file(edit_args)?;
    }

    Ok(())
}

fn edit_army_file(args: &EditArmyArgs) -> anyhow::Result<()> {
    let army_file: PathBuf = args.army_file.clone().into();

    // Load the army file.
    let file = File::open(army_file.clone())?;
    let army = Decoder::new(file).decode()?;

    // Serialize the army to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&army, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&army)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        army_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("army"),
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

    // Deserialize the modified string to an army.
    let modified_army = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified army to the original file.
    let file = File::create(army_file)?;
    Encoder::new(file).encode(&modified_army)?;

    println!("Army file successfully edited");

    Ok(())
}
