use crate::core::errors::RawstErr;

pub mod rawstcli_proto {
    tonic::include_proto!("rawstproto");
}

use crate::cli::args::rawstcli_proto::rawst_download_client::RawstDownloadClient;
use crate::cli::args::rawstcli_proto::Request as ProtoRequest;

pub async fn run_client(data: ProtoRequest, address: String) -> Result<(), RawstErr> {
    let mut client = RawstDownloadClient::connect(address).await.map_err(|_| RawstErr::Unreachable)?;

    let request = tonic::Request::new(data);

    let response = client.send(request).await.map_err(|_| RawstErr::BadRequest)?;

    println!("RESPONSE= {:?}", response);

    Ok(())

}