use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::gameflow::*;

use super::io::{read_input_to_string, write_output_string};

#[derive(Args)]
pub struct GameflowArgs {
    #[command(subcommand)]
    pub subcommand: Option<GameflowSubcommands>,
}

#[derive(Subcommand)]
pub enum GameflowSubcommands {
    Edit(EditGameflowArgs),
    /// Decode a binary gameflow file and print its serialized form.
    Dump(DumpGameflowArgs),
    /// Encode a serialized gameflow description into the binary format.
    Write(WriteGameflowArgs),
}

#[derive(Args)]
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

#[derive(Args)]
pub struct DumpGameflowArgs {
    /// The path to the gameflow file to read.
    #[arg(index = 1)]
    pub gameflow_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Serialization format.
    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteGameflowArgs {
    /// The destination binary gameflow file to write.
    #[arg(index = 1)]
    pub gameflow_file: String,

    /// Input path containing serialized data. Use "-" for stdin.
    #[arg(short, long)]
    pub input: String,

    /// Serialization format of the input.
    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Clone, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Json => f.write_str("json"),
            Format::Ron => f.write_str("ron"),
        }
    }
}

pub fn run(args: &GameflowArgs) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(GameflowSubcommands::Edit(edit_args)) => edit_gameflow_file(edit_args)?,
        Some(GameflowSubcommands::Dump(dump_args)) => dump_gameflow_file(dump_args)?,
        Some(GameflowSubcommands::Write(write_args)) => write_gameflow_file(write_args)?,
        None => {}
    }

    Ok(())
}

fn dump_gameflow_file(args: &DumpGameflowArgs) -> anyhow::Result<()> {
    let file = File::open(&args.gameflow_file)?;
    let gameflow = Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => ron::ser::to_string_pretty(&gameflow, ron::ser::PrettyConfig::default())?,
        Format::Json => serde_json::to_string_pretty(&gameflow)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn write_gameflow_file(args: &WriteGameflowArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let gameflow: Gameflow = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.gameflow_file)?;
    Encoder::new(file).encode(&gameflow)?;

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
