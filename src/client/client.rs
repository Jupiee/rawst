use crate::core::errors::RawstErr;

pub mod rawstcli_proto {
    tonic::include_proto!("rawstproto");
}

use rawstcli_proto::rawst_download_client::RawstDownloadClient;
use rawstcli_proto::{DownloadArgs as ProtoDownloadArgs, Request as ProtoRequest};

pub async fn run_client(data: (Option<u32>, Option<String>, Vec<String>, Option<String>, String, Option<String>, Option<String>), address: String) -> Result<(), RawstErr> {
    let mut client = RawstDownloadClient::connect(address).await.map_err(|_| RawstErr::Unreachable)?;

    let download_args = ProtoDownloadArgs {
        threads: data.0,
        input: data.1,
        output_file_path: data.2,
        headers_file_path: data.3
    };

    let request_args = ProtoRequest {
        download_args: Some(download_args),
        color: data.4,
        log_verbosity: data.6,
        verbosity: data.5
    };

    let request = tonic::Request::new(request_args);

    let response = client.send(request).await.map_err(|_| RawstErr::BadRequest)?;

    println!("RESPONSE= {:?}", response);

    Ok(())

}