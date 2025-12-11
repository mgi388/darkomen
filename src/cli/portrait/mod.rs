use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::portrait::heads::*;

#[derive(Args)]
pub struct PortraitArgs {
    #[command(subcommand)]
    pub subcommand: Option<PortraitSubcommands>,
}

#[derive(Subcommand)]
pub enum PortraitSubcommands {
    EditHeads(EditHeadsArgs),
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

#[derive(Clone, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

pub fn run(args: &PortraitArgs) -> anyhow::Result<()> {
    if let Some(PortraitSubcommands::EditHeads(edit_args)) = &args.subcommand {
        edit_heads_file(edit_args)?;
    }

    Ok(())
}

fn edit_heads_file(args: &EditHeadsArgs) -> anyhow::Result<()> {
    let heads_file: PathBuf = args.heads_file.clone().into();

    // Load the heads database file.
    let file = File::open(heads_file.clone())?;
    let heads_db = Decoder::new(file).decode()?;

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
    Encoder::new(file).encode(&modified_heads_db)?;

    println!("Heads database file successfully edited");

    Ok(())
}
