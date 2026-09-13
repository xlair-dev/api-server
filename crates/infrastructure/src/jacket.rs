use std::time::Duration;

use aws_sdk_s3::{Client, presigning::PresigningConfig};
use thiserror::Error;

const UPLOAD_EXPIRATION: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Error)]
pub enum JacketStorageError {
    #[error("failed to create presigned URL: {0}")]
    Presigning(String),
}

#[derive(Clone)]
pub struct JacketStorage {
    client: Client,
    bucket: String,
    public_base_url: String,
}

impl JacketStorage {
    pub async fn new(
        endpoint: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        public_base_url: String,
    ) -> Self {
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "xlair-r2",
        );
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(aws_sdk_s3::config::Region::new("auto"))
            .credentials_provider(credentials)
            .load()
            .await;
        Self {
            client: Client::new(&config),
            bucket,
            public_base_url: public_base_url.trim_end_matches('/').to_owned(),
        }
    }

    pub async fn create_upload_url(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<(String, String), JacketStorageError> {
        let config = PresigningConfig::expires_in(UPLOAD_EXPIRATION)
            .map_err(|error| JacketStorageError::Presigning(error.to_string()))?;
        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(config)
            .await
            .map_err(|error| JacketStorageError::Presigning(error.to_string()))?;
        Ok((
            request.uri().to_owned(),
            format!("{}/{}", self.public_base_url, key),
        ))
    }
}
