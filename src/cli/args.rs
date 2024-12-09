use std::path::PathBuf;

use directories::BaseDirs;
use iri_string::types::IriString;

use clap::Args;
use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;
use clap_complete::Generator;
use clap_complete::Shell;
use clap_num::number_range;

// Commands
// ========

// NOTE: Misleadingly, Command is a clap Subcommand underneath.
//       We pretend it's just the command outside this file as
//       the implementation details shouldn't be leaked.
/// The Rawst command.
///
/// - Download
/// - Resume
/// - History
#[derive(Subcommand, Debug, PartialEq)]
#[command(name = "rawst-subcommand")]
pub enum Command {
    /// Download files
    Download(DownloadArgs),
    /// Resume partial downloads
    Resume(ResumeArgs),
    /// View download history
    History(HistoryArgs),
}

// Subcommands
// -----------

// Download
const MAX_DOWNLOAD_THREADS: u8 = 8;

#[derive(Args, Debug, PartialEq)]
pub struct DownloadArgs {
    // Configuration
    /// Maximum amount of threads used to download
    ///
    /// Limited to 8 threads to avoid throttling
    #[arg(
      short,
      long,
      default_value_t=MAX_DOWNLOAD_THREADS,
      value_parser=limit_max_download_threads
    )]
    pub threads: u8,

    // Inputs
    // TODO: merge iris arg and input_file arg into one arg
    /// File where to look for download IRIs
    #[arg(short, long, default_value=None)]
    pub input_file: Option<PathBuf>,

    /// The IRIs to download
    #[arg()]
    pub iris: Vec<IriString>,

    // Outputs
    /// The path to the downloaded files
    #[arg(long)]
    pub output_file_path: Vec<PathBuf>,
}

fn limit_max_download_threads(s: &str) -> Result<u8, String> {
    number_range(s, 0, MAX_DOWNLOAD_THREADS)
}

// Resume
#[derive(Args, Debug, PartialEq)]
pub struct ResumeArgs {
    /// The Downloads to resume
    ///
    /// TODO: Default to resume the last download
    #[arg(default_value="auto")]
    pub download_ids: Vec<String>,
}

// History
#[derive(Args, Debug, PartialEq)]
pub struct HistoryArgs {}

/// Actual struct handled by clap
///
/// Not really what we want to use directly as it has extra noise,
/// - Autocompletion generation
/// - Version
/// - About
/// - Default subcommand
#[derive(Parser, Debug, PartialEq)]
#[command(name = "rawst", version, about, long_about = None)]
#[clap(color = concolor_clap::color_choice())]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(short, long)]
    pub verbosity: Option<log::LevelFilter>,
    #[arg(long)]
    pub log_verbosity: Option<log::LevelFilter>,

    #[command(flatten)]
    pub color: concolor_clap::Color,

    // Implementation details
    // ----------------------

    // Hack to default to `rawst download ...`
    // The setup to make Download the default subcommand come from,
    // - https://github.com/clap-rs/clap/discussions/4134#discussioncomment-3511528
    #[command(flatten)]
    default_command: Option<DownloadArgs>,

    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
}

fn generate_completion_script<G: Generator>(gen: G, cmd: &mut clap::Command) {
    let cmd_name = cmd.get_name().to_string();
    let base_dirs = BaseDirs::new().unwrap();
    let config_dir = base_dirs.config_dir().join("rawst").to_path_buf();
    clap_complete::generate_to(gen, cmd, cmd_name, &config_dir).unwrap();
    println!("Generated completion script at {}", config_dir.display())
}

pub fn get() -> Arguments {
    let mut args = Arguments::parse();

    if let Some(default_command_args) = args.default_command {
        args.default_command = None;
        args.command = Some(Command::Download(default_command_args));
    }

    if let Some(generator) = args.generator {
        let mut cmd = Arguments::command();
        eprintln!("Generating completion file for {generator:?}...");
        generate_completion_script(generator, &mut cmd);

        args.command = None;
        args.default_command = None;
    }

    args
}
