use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::gameflow::*;

#[derive(Debug, Args)]
pub struct GameflowArgs {
    #[command(subcommand)]
    pub subcommand: Option<GameflowSubcommands>,
}

#[derive(Debug, Subcommand)]
pub enum GameflowSubcommands {
    Edit(EditGameflowArgs),
}

#[derive(Debug, Args)]
pub struct EditGameflowArgs {
    /// The path to the gameflow file to edit, e.g., ".../CH1_ALL.DOT".
    #[arg(index = 1)]
    pub gameflow_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the gameflow file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

pub fn run(args: &GameflowArgs) -> anyhow::Result<()> {
    if let Some(GameflowSubcommands::Edit(edit_args)) = &args.subcommand {
        edit_gameflow_file(edit_args)?;
    }

    Ok(())
}

fn edit_gameflow_file(args: &EditGameflowArgs) -> anyhow::Result<()> {
    let gameflow_file: PathBuf = args.gameflow_file.clone().into();

    // Load the gameflow file.
    let file = File::open(gameflow_file.clone())?;
    let gameflow = Decoder::new(file).decode()?;

    // Serialize the gameflow to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&gameflow, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&gameflow)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        gameflow_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("gameflow"),
    );
    let suffix = format!(".{}", extension);
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

    // Deserialize the modified string to an gameflow.
    let modified_gameflow = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified gameflow to the original file.
    let file = File::create(gameflow_file)?;
    Encoder::new(file).encode(&modified_gameflow)?;

    println!("Gameflow file successfully edited");

    Ok(())
}
