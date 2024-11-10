use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;
use iri_string::types::IriString;
use reqwest::{
    header::{HeaderMap, HeaderValue, RANGE},
    Client, StatusCode,
};

use crate::core::config::Config;
use crate::core::errors::RawstErr;
use crate::core::io::{create_cache, create_file, merge_files};
use crate::core::task::{ChunkType, HttpTask};

#[derive(Clone, Default)]
pub struct HttpHandler {
    pub client: Client,
}

impl HttpHandler {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn sequential_download(
        &self,
        task: &HttpTask,
        progressbar: &ProgressBar,
        config: &Config,
    ) -> Result<(), RawstErr> {
        let mut headers = HeaderMap::new();

        if let ChunkType::Single(chunk) = &task.chunk_data {
            let range_value = format!("bytes={}-{}", chunk.x_offset, chunk.y_offset);

            headers.insert(RANGE, HeaderValue::from_str(range_value.as_str()).unwrap());
        }

        let response = self
            .client
            .get(to_reqwest_url(&task.iri))
            .headers(headers)
            .send()
            .await
            .map_err(RawstErr::HttpError)?;

        if response.status().is_success() {
            create_file(task, response, progressbar, &config.download_path).await?;
        }

        Ok(())
    }

    pub async fn concurrent_download(
        &self,
        task: &HttpTask,
        progressbar: &ProgressBar,
        config: &Config,
    ) -> Result<(), RawstErr> {
        // Creates a stream iter for downloading each chunk separately
        let download_tasks = stream::iter((0..config.threads).map(|i| {
            let client = &self.client;

            // Creates closure for each request and IO operation
            // Each closure has separate IO operation
            async move {
                if let ChunkType::Multiple(chunks) = &task.chunk_data {
                    let response = client
                        .get(to_reqwest_url(&task.iri))
                        .header(
                            RANGE,
                            format!("bytes={}-{}", chunks[i].x_offset, chunks[i].y_offset),
                        )
                        .send()
                        .await
                        .map_err(RawstErr::HttpError)?;

                    if response.status().is_success() {
                        create_cache(i, task, response, progressbar, &config.cache_path).await?;
                    }
                }

                Ok::<_, RawstErr>(())
            }
        }));

        download_tasks
            .buffer_unordered(config.threads)
            .collect::<Vec<_>>()
            .await;

        merge_files(&task.filename, config).await?;

        Ok(())
    }

    pub async fn cache_headers(&self, iri: &IriString) -> Result<HeaderMap, RawstErr> {
        let response = self
            .client
            .head(to_reqwest_url(iri))
            .send()
            .await
            .map_err(|_| RawstErr::Unreachable)?;

        match response.status() {
            StatusCode::OK => Ok(response.headers().to_owned()),

            StatusCode::BAD_REQUEST => Err(RawstErr::BadRequest),
            StatusCode::UNAUTHORIZED => Err(RawstErr::Unauthorized),
            StatusCode::FORBIDDEN => Err(RawstErr::Forbidden),
            StatusCode::NOT_FOUND => Err(RawstErr::NotFound),
            StatusCode::INTERNAL_SERVER_ERROR => Err(RawstErr::InternalServerError),

            _ => Err(RawstErr::Unknown(
                response.error_for_status().err().unwrap(),
            )),
        }
    }
}

/// Converts a IriString into reqwest::Url (url::Url)
fn to_reqwest_url(iri: &IriString) -> reqwest::Url {
    let uri: iri_string::types::UriString = iri.clone().encode_into_uri();

    reqwest::Url::parse(uri.as_str()).unwrap()
}
