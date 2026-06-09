use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::m3d::*;

use super::io::{read_input_to_string, write_output_string};

#[derive(Args)]
pub struct M3dArgs {
    #[command(subcommand)]
    pub subcommand: Option<M3dSubcommands>,
}

#[derive(Subcommand)]
pub enum M3dSubcommands {
    Edit(EditM3dArgs),
    /// Decode a binary M3D file and print its serialized form.
    Dump(DumpM3dArgs),
    /// Encode a serialized M3D description into the binary format.
    Write(WriteM3dArgs),
}

#[derive(Args)]
pub struct EditM3dArgs {
    /// The path to the M3D/M3X file to edit, e.g., ".../BASE.M3D".
    #[arg(index = 1)]
    pub m3d_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the M3D file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct DumpM3dArgs {
    /// The path to the M3D/M3X file to read.
    #[arg(index = 1)]
    pub m3d_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Serialization format.
    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteM3dArgs {
    /// The destination binary M3D file to write.
    #[arg(index = 1)]
    pub m3d_file: String,

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

pub fn run(args: &M3dArgs) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(M3dSubcommands::Edit(edit_args)) => edit_m3d_file(edit_args)?,
        Some(M3dSubcommands::Dump(dump_args)) => dump_m3d_file(dump_args)?,
        Some(M3dSubcommands::Write(write_args)) => write_m3d_file(write_args)?,
        None => {}
    }

    Ok(())
}

fn dump_m3d_file(args: &DumpM3dArgs) -> anyhow::Result<()> {
    let file = File::open(&args.m3d_file)?;
    let m3d = Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => ron::ser::to_string_pretty(&m3d, ron::ser::PrettyConfig::default())?,
        Format::Json => serde_json::to_string_pretty(&m3d)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn write_m3d_file(args: &WriteM3dArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let m3d: M3d = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.m3d_file)?;
    Encoder::new(file).encode(&m3d)?;

    Ok(())
}

fn edit_m3d_file(args: &EditM3dArgs) -> anyhow::Result<()> {
    let m3d_file: PathBuf = args.m3d_file.clone().into();

    // Load the M3D file.
    let file = File::open(m3d_file.clone())?;
    let m3d = Decoder::new(file).decode()?;

    // Serialize the M3D to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&m3d, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&m3d)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        m3d_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("m3d"),
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

    // Deserialize the modified string to an M3D.
    let modified_m3d = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified M3D to the original file.
    let file = File::create(m3d_file)?;
    Encoder::new(file).encode(&modified_m3d)?;

    println!("M3D file successfully edited");

    Ok(())
}
