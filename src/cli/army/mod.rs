use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand};
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
    /// The path to the army file to edit, e.g. ".../B1_01/B101MRC.ARM".
    #[arg(index = 1)]
    pub army_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,
}

pub fn run(args: &ArmyArgs) -> anyhow::Result<()> {
    if let Some(ArmySubcommands::Edit(edit_args)) = &args.subcommand {
        edit_army_file(edit_args.editor.clone(), edit_args.army_file.clone().into())?;
    }

    Ok(())
}

fn edit_army_file(editor: String, file_path: PathBuf) -> anyhow::Result<()> {
    // Load the army file.
    let file = File::open(file_path.clone())?;
    let army = Decoder::new(file).decode()?;

    // Serialize the army to RON.
    let ron_string = ron::ser::to_string_pretty(&army, ron::ser::PrettyConfig::default())?;

    // Write the RON string to a temporary file.
    let prefix = file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("army")
        .to_owned()
        + ".";
    let mut temp_file = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(".ron")
        .tempfile()?;
    temp_file.write_all(ron_string.as_bytes())?;
    temp_file.flush()?;

    // Open the temporary file in the editor.
    let (editor, editor_args) = {
        let mut parts = editor.split_whitespace();
        let editor = parts.next().unwrap();
        let editor_args = parts.collect::<Vec<_>>();
        (editor, editor_args)
    };
    let mut command = std::process::Command::new(editor);
    command.args(editor_args);
    // This call blocks until the editor process exits.
    command.arg(temp_file.path()).status()?;

    // Read the modified RON string from the temporary file.
    let mut modified_ron_string = String::new();
    temp_file
        .reopen()?
        .read_to_string(&mut modified_ron_string)?;

    // Deserialize the modified RON string to an army.
    let modified_army = ron::de::from_str(&modified_ron_string)?;

    // Write the modified army to the original file.
    let file = File::create(file_path)?;
    Encoder::new(file).encode(&modified_army)?;

    Ok(())
}
