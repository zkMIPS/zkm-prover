use anyhow::anyhow;
use std::io;
use std::path::Path;
use tonic::transport::{Certificate, Identity};

#[derive(Clone)]
pub struct Config {
    pub ca_cert: Certificate,
    pub identity: Identity,
}

impl Config {
    pub async fn new(
        ca_cert_path: String,
        cert_path: String,
        key_path: String,
    ) -> anyhow::Result<Self> {
        let (ca_cert, identity) = get_cert_and_identity(ca_cert_path, cert_path, key_path).await?;
        Ok(Config { ca_cert, identity })
    }
}

async fn get_cert_and_identity(
    ca_cert_path: String,
    cert_path: String,
    key_path: String,
) -> anyhow::Result<(Certificate, Identity)> {
    let ca_cert_path = Path::new(&ca_cert_path);
    let cert_path = Path::new(&cert_path);
    let key_path = Path::new(&key_path);
    if !ca_cert_path.is_file() || !cert_path.is_file() || !key_path.is_file() {
        return Err(anyhow!(
            "both ca_cert_path, cert_path and key_path should be valid file"
        ));
    }

    let ca_cert = tokio::fs::read(ca_cert_path).await.map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("Failed to read {ca_cert_path:?}, err: {err}"),
        )
    })?;
    let ca_cert = Certificate::from_pem(ca_cert);

    let cert = tokio::fs::read(cert_path).await.map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("Failed to read {cert_path:?}, err: {err}"),
        )
    })?;
    let key = tokio::fs::read(key_path).await.map_err(|err| {
        io::Error::new(
            err.kind(),
            format!("Failed to read {key_path:?}, err: {err}"),
        )
    })?;
    let identity = Identity::from_pem(cert, key);

    Ok((ca_cert, identity))
}
