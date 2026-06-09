use std::{
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
};

use clap::{Args, Subcommand, ValueEnum};
use darkomen::project::*;

use super::io::{read_input_to_string, write_output_string};

#[derive(Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub subcommand: Option<ProjectSubcommands>,
}

#[derive(Subcommand)]
pub enum ProjectSubcommands {
    Edit(EditProjectArgs),
    /// Decode a binary project file and print its serialized form.
    Dump(DumpProjectArgs),
    /// Encode a serialized project description into the binary format.
    Write(WriteProjectArgs),
}

#[derive(Args)]
pub struct EditProjectArgs {
    /// The path to the project file to edit, e.g., ".../B1_01/B1_01.PRJ".
    #[arg(index = 1)]
    pub project_file: String,

    /// The name of the text editor to use.
    #[arg(short, long, default_value = "code --wait")]
    pub editor: String,

    /// The format to edit the project file in.
    #[arg(short, long, default_value_t=Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct DumpProjectArgs {
    /// The path to the project file to read.
    #[arg(index = 1)]
    pub project_file: String,

    /// Output path. Use "-" or omit for stdout.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Serialization format.
    #[arg(short, long, default_value_t = Format::Json)]
    #[clap(value_enum)]
    pub format: Format,
}

#[derive(Args)]
pub struct WriteProjectArgs {
    /// The destination binary project file to write.
    #[arg(index = 1)]
    pub project_file: String,

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

pub fn run(args: &ProjectArgs) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(ProjectSubcommands::Edit(edit_args)) => edit_project_file(edit_args)?,
        Some(ProjectSubcommands::Dump(dump_args)) => dump_project_file(dump_args)?,
        Some(ProjectSubcommands::Write(write_args)) => write_project_file(write_args)?,
        None => {}
    }

    Ok(())
}

fn dump_project_file(args: &DumpProjectArgs) -> anyhow::Result<()> {
    let file = File::open(&args.project_file)?;
    let project = Decoder::new(file).decode()?;

    let as_string = match args.format {
        Format::Ron => ron::ser::to_string_pretty(&project, ron::ser::PrettyConfig::default())?,
        Format::Json => serde_json::to_string_pretty(&project)?,
    };

    write_output_string(args.output.as_deref(), &as_string)
}

fn write_project_file(args: &WriteProjectArgs) -> anyhow::Result<()> {
    let s = read_input_to_string(&args.input)?;
    let project: Project = match args.format {
        Format::Ron => ron::de::from_str(&s)?,
        Format::Json => serde_json::from_str(&s)?,
    };

    let file = File::create(&args.project_file)?;
    Encoder::new(file).encode(&project)?;

    Ok(())
}

fn edit_project_file(args: &EditProjectArgs) -> anyhow::Result<()> {
    let project_file: PathBuf = args.project_file.clone().into();

    // Load the project file.
    let file = File::open(project_file.clone())?;
    let project = Decoder::new(file).decode()?;

    // Serialize the project to a human-readable string.
    let (as_string, extension) = match args.format {
        Format::Ron => (
            ron::ser::to_string_pretty(&project, ron::ser::PrettyConfig::default())?,
            "ron",
        ),
        Format::Json => (serde_json::to_string_pretty(&project)?, "json"),
    };

    // Write the human-readable string to a temporary file.
    let prefix = format!(
        "{}.",
        project_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("project"),
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

    // Deserialize the modified string to an project.
    let modified_project = match args.format {
        Format::Ron => ron::de::from_str(&modified_string)?,
        Format::Json => serde_json::from_str(&modified_string)?,
    };

    // Write the modified project to the original file.
    let file = File::create(project_file)?;
    Encoder::new(file).encode(&modified_project)?;

    println!("Project file successfully edited");

    Ok(())
}
