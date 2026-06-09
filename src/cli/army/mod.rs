use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::army::*;

use super::io::{read_input_to_string, write_output_string};

#[derive(Args)]
pub struct ArmyArgs {
    #[command(subcommand)]
    pub subcommand: Option<ArmySubcommands>,
}

#[derive(Subcommand)]
pub enum ArmySubcommands {
    Edit(EditArmyArgs),
    /// Decode a binary army file and print its serialized form.
    Dump(DumpArmyArgs),
    /// Encode a serialized army description into the binary format.
    Write(WriteArmyArgs),
}

#[derive(Args)]
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

#[derive(Args)]
pub struct DumpArmyArgs {
    /// The path to the army file to read.
    #[arg(index = 1)]
    pub army_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Serialization format.
    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteArmyArgs {
    /// The destination binary army file to write.
    #[arg(index = 1)]
    pub army_file: String,

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

pub fn run(args: &ArmyArgs) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(ArmySubcommands::Edit(edit_args)) => edit_army_file(edit_args)?,
        Some(ArmySubcommands::Dump(dump_args)) => dump_army_file(dump_args)?,
        Some(ArmySubcommands::Write(write_args)) => write_army_file(write_args)?,
        None => {}
    }

    Ok(())
}

fn dump_army_file(args: &DumpArmyArgs) -> anyhow::Result<()> {
    let file = File::open(&args.army_file)?;
    let army = Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => ron::ser::to_string_pretty(&army, ron::ser::PrettyConfig::default())?,
        Format::Json => serde_json::to_string_pretty(&army)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn write_army_file(args: &WriteArmyArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let army: Army = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.army_file)?;
    Encoder::new(file).encode(&army)?;

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
