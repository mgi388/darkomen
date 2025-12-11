use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::m3d::*;

#[derive(Args)]
pub struct M3dArgs {
    #[command(subcommand)]
    pub subcommand: Option<M3dSubcommands>,
}

#[derive(Subcommand)]
pub enum M3dSubcommands {
    Edit(EditM3dArgs),
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

#[derive(Clone, ValueEnum)]
pub enum Format {
    Json,
    Ron,
}

pub fn run(args: &M3dArgs) -> anyhow::Result<()> {
    if let Some(M3dSubcommands::Edit(edit_args)) = &args.subcommand {
        edit_m3d_file(edit_args)?;
    }

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
