use crate::cli::args::CommandArgs;
use crate::core::errors::RawstErr;

pub mod rawstcli_proto {
    tonic::include_proto!("rawstproto");
}

use rawstcli_proto::rawst_download_client::RawstDownloadClient;
use rawstcli_proto::{DownloadArgs as ProtoDownloadArgs, ResumeArgs as ProtoResumeArgs, HistoryArgs as ProtoHistoryArgs, Request as ProtoRequest, request::CommandArgs as ProtoCommandArgs};

pub async fn run_client(data: (CommandArgs, String, Option<String>, Option<String>, String), address: String) -> Result<(), RawstErr> {
    let mut client = RawstDownloadClient::connect(address).await.map_err(|_| RawstErr::Unreachable)?;

    let command_args = match data.0 {
        CommandArgs::Download(args) => {
            ProtoCommandArgs::DownloadArgs(
                ProtoDownloadArgs {
                    threads: args.0,
                    input: args.1,
                    output_file_path: args.2,
                    headers_file_path: args.3
                }
            )
        },
        CommandArgs::History(args) => {
            ProtoCommandArgs::HistoryArgs(
                ProtoHistoryArgs {
                    show: args.0,
                    clear: args.1,
                }
            )
        },
        CommandArgs::Resume(args) => {
            ProtoCommandArgs::ResumeArgs(
                ProtoResumeArgs {
                    download_ids: args
                }
            )
        }

    };

    let request_args = ProtoRequest {
        argument_type: data.4,
        command_args: Some(command_args),
        color: data.1,
        log_verbosity: data.2,
        verbosity: data.3
    };

    let request = tonic::Request::new(request_args);

    let response = client.send(request).await.map_err(|_| RawstErr::BadRequest)?;

    println!("RESPONSE= {:?}", response);

    Ok(())

}