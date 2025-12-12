use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::portrait::heads;
use darkomen::portrait::keyframes;
use darkomen::portrait::sequences;

#[derive(Args)]
pub struct PortraitArgs {
    #[command(subcommand)]
    pub subcommand: Option<PortraitSubcommands>,
}

#[derive(Subcommand)]
#[expect(
    clippy::enum_variant_names,
    reason = "These are edit subcommands and we might have non-edit subcommands later."
)]
pub enum PortraitSubcommands {
    EditHeads(EditHeadsArgs),
    EditKeyframes(EditKeyframesArgs),
    EditSequences(EditSequencesArgs),
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

#[derive(Clone, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

pub fn run(args: &PortraitArgs) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(PortraitSubcommands::EditHeads(edit_args)) => edit_heads_file(edit_args)?,
        Some(PortraitSubcommands::EditKeyframes(edit_args)) => edit_keyframes_file(edit_args)?,
        Some(PortraitSubcommands::EditSequences(edit_args)) => edit_sequences_file(edit_args)?,
        None => {}
    }

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
