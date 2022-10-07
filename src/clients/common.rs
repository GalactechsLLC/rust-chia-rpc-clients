use reqwest::{Certificate, Client, ClientBuilder, Identity};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::time::Duration;

pub fn get_url(host: &str, port: u32, request_uri: &str) -> String {
    format!(
        "https://{host}:{port}/{request_uri}",
        host = host,
        port = port,
        request_uri = request_uri
    )
}

pub fn get_client(ssl_path: &str) -> Result<Client, Error> {
    let mut cert_buf = Vec::new();
    File::open(format!("{}/{}", ssl_path, "/daemon/private_daemon.crt"))?
        .read_to_end(&mut cert_buf)?;
    let cert = Certificate::from_pem(&cert_buf).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to parse Certificate file: {:?}", e),
        )
    })?;
    let mut key_buf = Vec::new();
    File::open(format!("{}/{}", ssl_path, "/daemon/private_daemon.key"))?
        .read_to_end(&mut key_buf)?;
    let id = Identity::from_pem(&key_buf).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to parse Identity file: {:?}", e),
        )
    })?;
    ClientBuilder::new()
        .add_root_certificate(cert)
        .identity(id)
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| Error::new(ErrorKind::Other, format!("{:?}", e)))
}

pub async fn post<T>(client: &Client, url: &str, data: &Map<String, Value>) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    match client.post(url).json(data).send().await {
        Ok(resp) => match resp.status() {
            reqwest::StatusCode::OK => {
                let body = resp
                    .text()
                    .await
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
                serde_json::from_str(body.as_str()).map_err(|e| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("Failed to Parse Json {},\r\n {}", body, e),
                    )
                })
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Bad Status Code: {:?}, for URL {:?}", resp.status(), url),
            )),
        },
        Err(err) => Err(Error::new(ErrorKind::InvalidData, format!("{:?}", err))),
    }
}
