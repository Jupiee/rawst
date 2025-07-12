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

pub mod rawstcli_proto {
    tonic::include_proto!("rawstproto");
}

use rawstcli_proto::{DownloadArgs as ProtoDownloadArgs, ResumeArgs as ProtoResumeArgs, HistoryArgs as ProtoHistoryArgs, Request as ProtoRequest, request::CommandArgs as ProtoCommandArgs};

#[derive(Debug, PartialEq, Clone)]
pub enum InputSource {
    File(PathBuf),
    Iris(Vec<IriString>)

}

fn parse_input_source(s: &str) -> Result<InputSource, String> {

    if s.ends_with(".txt") {
        Ok(InputSource::File(PathBuf::from(s)))

    } else {
        let iris = s.split(',')
            .map(|s| IriString::try_from(s).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(InputSource::Iris(iris))

    }

}

// Commands
// ========

// NOTE: Misleadingly, Command is a clap Subcommand underneath.
//       We pretend it's just the command outside this file as
//       the implementation details shouldn't be leaked.
/// The Rawst command.
///
/// - Server
/// - Download
/// - Resume
/// - History
#[derive(Subcommand, Debug, PartialEq)]
#[command(name = "rawst-subcommand")]
pub enum Command {
    /// Start the server
    Server(ServerOptions),
    /// Download files
    Download(DownloadArgs),
    /// Resume partial downloads
    Resume(ResumeArgs),
    /// Inspect download history
    History(HistoryArgs),
    /// Edit config settings
    Config,
}

// Subcommands
// -----------

// Server
#[derive(Args, Debug, PartialEq)]
pub struct ServerOptions {
    #[arg(long, default_value="127.0.0.1:50051")]
    pub server_address: String,

}

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
      value_parser=limit_max_download_threads
    )]
    pub threads: Option<u8>,

    // Inputs
    /// The input source to download from
    /// 
    /// URL and text file path that contains URLS could be used
    #[arg(value_parser=parse_input_source, default_value=None)]
    pub input: Option<InputSource>,

    // Outputs
    /// PATH where the files are downloaded along with custom file name
    /// 
    /// passing only custom file name without PATH will download the file with custom name in the default download directory
    /// 
    /// eg. `foo\bar\custom_name.exe` or `custom_name.exe`
    #[arg(long)]
    pub output_file_path: Vec<PathBuf>,

    /// Path to JSON file containing request headers.
    #[arg(long, default_value=None)]
    pub headers_file_path: Option<PathBuf>,
}

fn limit_max_download_threads(s: &str) -> Result<u8, String> {
    number_range(s, 0, MAX_DOWNLOAD_THREADS)
}

// Resume
#[derive(Args, Debug, PartialEq)]
pub struct ResumeArgs {
    /// The Downloads to resume
    #[arg(default_value="auto")]
    pub download_ids: Vec<String>,
}

#[derive(Args, Debug, PartialEq)]
pub struct HistoryArgs{
    /// Show the history records of all downloads
    #[arg(long, action)]
    pub show: bool,
    
    /// Clear all the records in history
    #[arg(long, action)]
    pub clear: bool

}

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

impl Arguments {

    pub fn from_primitive_types(proto: ProtoRequest) -> Self {
        let command_args = if let Some(proto_cmd_args) = proto.command_args {
            match proto_cmd_args {
                ProtoCommandArgs::DownloadArgs(args) => {

                    let threads = match args.threads {
                        Some(num_of_threads) => Some(num_of_threads as u8),
                        None => None
                    };

                    let input = match args.input {
                        Some(source_string) => {
                            let source = parse_input_source(&source_string.as_str()).unwrap();
                            Some(source)

                        },
                        None => None

                    };

                    let output_file_path = args.output_file_path.into_iter().map(|i| PathBuf::from(i)).collect::<Vec<PathBuf>>();

                    let headers_file_path = match args.headers_file_path {
                        Some(file_path_string) => Some(PathBuf::from(file_path_string)),
                        None => None

                    };

                    Some(Command::Download(
                        DownloadArgs {
                            threads,
                            input,
                            output_file_path,
                            headers_file_path
                        }
                    ))

                },
                ProtoCommandArgs::ResumeArgs(args) => {
                    Some(Command::Resume(
                        ResumeArgs {
                            download_ids: args.download_ids
                        }
                    ))

                },
                ProtoCommandArgs::HistoryArgs(args) => {
                    Some(Command::History(
                        HistoryArgs {
                            show: args.show,
                            clear: args.clear
                        }
                    ))

                }
                
            }

        } else {
            None

        };

        let auto = "Auto".to_owned();
        let always = "Always".to_owned();

        let color_choice = if proto.color == auto {
            concolor_clap::ColorChoice::Auto
            
        } else if proto.color == always {
            concolor_clap::ColorChoice::Always

        } else {
            concolor_clap::ColorChoice::Never

        };

        let color = concolor_clap::Color {
            color: color_choice
        };

        let off =  "Off".to_owned();
        let debug = "Debug".to_owned();
        let error = "Error".to_owned();
        let info = "Info".to_owned();
        let trace = "Trace".to_owned();

        let verbosity = match proto.verbosity {
            Some(level) => {

                if level == off {
                    Some(log::LevelFilter::Off)

                } else if level == debug {
                    Some(log::LevelFilter::Debug)

                } else if level == error {
                    Some(log::LevelFilter::Error)

                } else if level == info {
                    Some(log::LevelFilter::Info)

                } else if level == trace {
                    Some(log::LevelFilter::Trace)

                } else {
                    Some(log::LevelFilter::Warn)

                }

            },
            None => None

        };

        let log_verbosity = match proto.log_verbosity {
            Some(level) => {

                if level == off {
                    Some(log::LevelFilter::Off)

                } else if level == debug {
                    Some(log::LevelFilter::Debug)

                } else if level == error {
                    Some(log::LevelFilter::Error)

                } else if level == info {
                    Some(log::LevelFilter::Info)

                } else if level == trace {
                    Some(log::LevelFilter::Trace)

                } else {
                    Some(log::LevelFilter::Warn)

                }

            },
            None => None

        };

        Arguments {
            command: command_args,
            verbosity,
            log_verbosity,
            color,
            default_command: None,
            generator: None
        }

    }

    pub fn to_primitive_types(self) -> ProtoRequest {

        let command_type: String;

        let command_args = match self.command {
            Some(Command::Download(download_args)) => {
                
                let threads = match download_args.threads {
                    Some(number) => Some(number as u32),
                    None => None
                };

                let string_url = match download_args.input {
                    Some(InputSource::File(file_pathbuf)) => Some(file_pathbuf.to_string_lossy().into_owned()) ,
                    Some(InputSource::Iris(vec_iri)) => Some(vec_iri.into_iter().next().unwrap().as_str().to_string()),
                    _ => None
                };

                let vec_output_pathbuf = download_args.output_file_path.into_iter().map(|i| i.to_string_lossy().into_owned()).collect::<Vec<String>>();
                let header_file_path = match download_args.headers_file_path {
                    Some(file_path) => Some(file_path.to_string_lossy().into_owned()),
                    None => None
                };

                command_type = "Download".to_string();

                ProtoCommandArgs::DownloadArgs(
                    ProtoDownloadArgs {
                        threads,
                        input: string_url,
                        output_file_path: vec_output_pathbuf,
                        headers_file_path: header_file_path
                    }
                )

            },
            Some(Command::Resume(resume_args)) => {
                command_type = "Resume".to_string();

                ProtoCommandArgs::ResumeArgs(
                    ProtoResumeArgs {
                        download_ids: resume_args.download_ids
                    }
                )

            },
            Some(Command::History(history_args)) => {
                command_type = "History".to_string();

                ProtoCommandArgs::HistoryArgs(
                    ProtoHistoryArgs {
                        show: history_args.show,
                        clear: history_args.clear,
                    }
                )

            },
            _ => {
                command_type = "Download".to_string();
                ProtoCommandArgs::DownloadArgs(
                    ProtoDownloadArgs {
                        threads: Some(0 as u32),
                        input: Some("".to_owned()),
                        output_file_path: vec!["".to_owned()],
                        headers_file_path: Some("".to_owned())
                    }
                )

            }

        };

        let color = match self.color.color {

            concolor_clap::ColorChoice::Auto => "Auto".to_owned(),
            concolor_clap::ColorChoice::Always => "Always".to_owned(),
            concolor_clap::ColorChoice::Never => "Never".to_owned(),

        };
        let verbosity = match self.verbosity {
            Some(log::LevelFilter::Off) => Some("Off".to_owned()),
            Some(log::LevelFilter::Debug) => Some("Debug".to_owned()),
            Some(log::LevelFilter::Error) => Some("Error".to_owned()),
            Some(log::LevelFilter::Info) => Some("Info".to_owned()),
            Some(log::LevelFilter::Trace) => Some("Trace".to_owned()),
            Some(log::LevelFilter::Warn) => Some("Warn".to_owned()),
            None => None

        };

        let log_verbosity = match self.log_verbosity {
            Some(log::LevelFilter::Off) => Some("Off".to_owned()),
            Some(log::LevelFilter::Debug) => Some("Debug".to_owned()),
            Some(log::LevelFilter::Error) => Some("Error".to_owned()),
            Some(log::LevelFilter::Info) => Some("Info".to_owned()),
            Some(log::LevelFilter::Trace) => Some("Trace".to_owned()),
            Some(log::LevelFilter::Warn) => Some("Warn".to_owned()),
            None => None

        };

        ProtoRequest {
            argument_type: command_type,
            command_args: Some(command_args),
            color,
            log_verbosity,
            verbosity
        }

    }

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
