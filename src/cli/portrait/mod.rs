use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::portrait::heads;
use darkomen::portrait::keyframes;
use darkomen::portrait::sequences;

use super::io::{read_input_to_string, write_output_string};

#[derive(Args)]
pub struct PortraitArgs {
    #[command(subcommand)]
    pub subcommand: Option<PortraitSubcommands>,
}

#[derive(Subcommand)]
pub enum PortraitSubcommands {
    EditHeads(EditHeadsArgs),
    EditKeyframes(EditKeyframesArgs),
    EditSequences(EditSequencesArgs),
    DumpHeads(DumpHeadsArgs),
    DumpKeyframes(DumpKeyframesArgs),
    DumpSequences(DumpSequencesArgs),
    WriteHeads(WriteHeadsArgs),
    WriteKeyframes(WriteKeyframesArgs),
    WriteSequences(WriteSequencesArgs),
}

#[derive(Args)]
pub struct EditHeadsArgs {
    /// The path to the heads database file to edit, e.g., ".../HEADS.DB".
    #[arg(index = 1)]
    pub heads_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the heads database file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct EditKeyframesArgs {
    /// The path to the keyframes file to edit, e.g., ".../0.KEY".
    #[arg(index = 1)]
    pub keyframes_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the keyframes file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct EditSequencesArgs {
    /// The path to the sequences file to edit, e.g., ".../0.SEQ".
    #[arg(index = 1)]
    pub sequences_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the sequences file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct DumpHeadsArgs {
    /// The path to the heads database file to read, e.g., ".../HEADS.DB".
    #[arg(index = 1)]
    pub heads_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct DumpKeyframesArgs {
    /// The path to the keyframes file to read, e.g., ".../0.KEY".
    #[arg(index = 1)]
    pub keyframes_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct DumpSequencesArgs {
    /// The path to the sequences file to read, e.g., ".../0.SEQ".
    #[arg(index = 1)]
    pub sequences_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteHeadsArgs {
    /// The destination binary heads database file to write.
    #[arg(index = 1)]
    pub heads_file: String,

    /// Input path containing serialized data. Use "-" for stdin.
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteKeyframesArgs {
    /// The destination binary keyframes file to write.
    #[arg(index = 1)]
    pub keyframes_file: String,

    /// Input path containing serialized data. Use "-" for stdin.
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteSequencesArgs {
    /// The destination binary sequences file to write.
    #[arg(index = 1)]
    pub sequences_file: String,

    /// Input path containing serialized data. Use "-" for stdin.
    #[arg(short, long)]
    pub input: String,

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

pub fn run(args: &PortraitArgs) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(PortraitSubcommands::EditHeads(edit_args)) => edit_heads_file(edit_args)?,
        Some(PortraitSubcommands::EditKeyframes(edit_args)) => edit_keyframes_file(edit_args)?,
        Some(PortraitSubcommands::EditSequences(edit_args)) => edit_sequences_file(edit_args)?,
        Some(PortraitSubcommands::DumpHeads(dump_args)) => dump_heads_file(dump_args)?,
        Some(PortraitSubcommands::DumpKeyframes(dump_args)) => dump_keyframes_file(dump_args)?,
        Some(PortraitSubcommands::DumpSequences(dump_args)) => dump_sequences_file(dump_args)?,
        Some(PortraitSubcommands::WriteHeads(write_args)) => write_heads_file(write_args)?,
        Some(PortraitSubcommands::WriteKeyframes(write_args)) => write_keyframes_file(write_args)?,
        Some(PortraitSubcommands::WriteSequences(write_args)) => write_sequences_file(write_args)?,
        None => {}
    }

    Ok(())
}

fn dump_heads_file(args: &DumpHeadsArgs) -> anyhow::Result<()> {
    let file = File::open(&args.heads_file)?;
    let heads_db = heads::Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => ron::ser::to_string_pretty(&heads_db, ron::ser::PrettyConfig::default())?,
        Format::Json => serde_json::to_string_pretty(&heads_db)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn dump_keyframes_file(args: &DumpKeyframesArgs) -> anyhow::Result<()> {
    let file = File::open(&args.keyframes_file)?;
    let keyframes_db = keyframes::Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => {
            ron::ser::to_string_pretty(&keyframes_db, ron::ser::PrettyConfig::default())?
        }
        Format::Json => serde_json::to_string_pretty(&keyframes_db)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn dump_sequences_file(args: &DumpSequencesArgs) -> anyhow::Result<()> {
    let file = File::open(&args.sequences_file)?;
    let sequences_db = sequences::Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => {
            ron::ser::to_string_pretty(&sequences_db, ron::ser::PrettyConfig::default())?
        }
        Format::Json => serde_json::to_string_pretty(&sequences_db)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn write_heads_file(args: &WriteHeadsArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let heads_db: heads::HeadsDatabase = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.heads_file)?;
    heads::Encoder::new(file).encode(&heads_db)?;

    Ok(())
}

fn write_keyframes_file(args: &WriteKeyframesArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let keyframes_db: keyframes::Keyframes = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.keyframes_file)?;
    keyframes::Encoder::new(file).encode(&keyframes_db)?;

    Ok(())
}

fn write_sequences_file(args: &WriteSequencesArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let sequences_db: sequences::Sequences = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.sequences_file)?;
    sequences::Encoder::new(file).encode(&sequences_db)?;

    Ok(())
}

fn edit_heads_file(args: &EditHeadsArgs) -> anyhow::Result<()> {
    let heads_file: PathBuf = args.heads_file.clone().into();

    // Load the heads database file.
    let file = File::open(heads_file.clone())?;
    let heads_db = heads::Decoder::new(file).decode()?;

    // Serialize the heads database to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&heads_db, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&heads_db)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        heads_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("heads"),
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

    // Deserialize the modified string to a heads database.
    let modified_heads_db = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified heads database to the original file.
    let file = File::create(heads_file)?;
    heads::Encoder::new(file).encode(&modified_heads_db)?;

    println!("Heads database file successfully edited");

    Ok(())
}

fn edit_keyframes_file(args: &EditKeyframesArgs) -> anyhow::Result<()> {
    let keyframes_file: PathBuf = args.keyframes_file.clone().into();

    // Load the keyframes file.
    let file = File::open(keyframes_file.clone())?;
    let keyframes_db = keyframes::Decoder::new(file).decode()?;

    // Serialize the keyframes database to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&keyframes_db, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&keyframes_db)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        keyframes_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("keyframes"),
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

    // Deserialize the modified string to a keyframes database.
    let modified_keyframes_db = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified keyframes database to the original file.
    let file = File::create(keyframes_file)?;
    keyframes::Encoder::new(file).encode(&modified_keyframes_db)?;

    println!("Keyframes file successfully edited");

    Ok(())
}

fn edit_sequences_file(args: &EditSequencesArgs) -> anyhow::Result<()> {
    let sequences_file: PathBuf = args.sequences_file.clone().into();

    // Load the sequences file.
    let file = File::open(sequences_file.clone())?;
    let sequences_db = sequences::Decoder::new(file).decode()?;

    // Serialize the sequences database to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&sequences_db, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&sequences_db)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        sequences_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("sequences"),
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

    // Deserialize the modified string to a sequences database.
    let modified_sequences_db = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified sequences database to the original file.
    let file = File::create(sequences_file)?;
    sequences::Encoder::new(file).encode(&modified_sequences_db)?;

    println!("Sequences file successfully edited");

    Ok(())
}
