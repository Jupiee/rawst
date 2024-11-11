use std::path::PathBuf;

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
    #[arg()]
    pub download_ids: Vec<String>,
}

// History
#[derive(Args, Debug, PartialEq)]
pub struct HistoryArgs {}

// Implementation details
// ----------------------

/// Actual struct handled by clap
///
/// Not really what we want to use directly as it has extra noise,
/// - Autocompletion generation
/// - Version
/// - About
/// - Default subcommand
#[derive(Parser, Debug, PartialEq)]
#[command(name = "rawst", version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,

    // Hack to default to `rawst download ...`
    // The setup to make Download the default subcommand come from,
    // - https://github.com/clap-rs/clap/discussions/4134#discussioncomment-3511528
    #[command(flatten)]
    default_command: Option<DownloadArgs>,

    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    let cmd_name = cmd.get_name().to_string();
    clap_complete::generate(gen, cmd, cmd_name, &mut std::io::stdout());
}

pub fn get_command() -> Option<Command> {
    let args = Arguments::parse();

    if let Some(generator) = args.generator {
        let mut cmd = Arguments::command();
        eprintln!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);

        None
    } else if let Some(default_download) = args.default_command {
        Some(Command::Download(default_download))
    } else if let Some(ref _command) = args.command {
        args.command
    } else {
        None
    }
}
