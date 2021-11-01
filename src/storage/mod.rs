mod config;

use anyhow::{bail, Error, Result};
pub use config::*;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};

use crate::EntropyConfig;

fn get_creds(config: &StorageConfig) -> Result<(Credentials, Region)> {
    let credentials = Credentials::new(
        Some(&config.credentials.access_key),
        Some(&config.credentials.secret_key),
        None,
        None,
        None,
    )?;
    let region = s3::Region::Custom {
        endpoint: config.credentials.endpoint.clone(),
        region: config
            .credentials
            .region
            .clone()
            .unwrap_or_else(|| "".to_string()),
    };

    Ok((credentials, region))
}

// Create minio bucket to store dynamic assets
pub async fn create_assets_bucket() -> anyhow::Result<()> {
    let config = EntropyConfig::load()?.storage;

    let bucket_config = BucketConfiguration::default();
    let (creds, region) = get_creds(&config)?;

    let res =
        Bucket::create_with_path_style(&config.assets_bucket, region, creds, bucket_config).await?;

    match res.response_code {
        409 => {
            debug!(
                "Assets bucket already exists. Moving on. [bucket={}]",
                config.assets_bucket
            );
        }
        200 => {
            info!("Created assets bucket: [bucket={}]", config.assets_bucket);
        }
        _ => {
            bail!(
                "S3 Response Code: {}\nS3 Response: {:?}",
                res.response_code,
                res.response_text
            );
        }
    }

    Ok(())
}

pub async fn get_asset(path: &str) -> Result<Vec<u8>> {
    let config = EntropyConfig::load()?.storage;
    let (creds, region) = get_creds(&config)?;
    let bucket = Bucket::new_with_path_style(&config.assets_bucket, region, creds)?;

    let (data, code) = bucket.get_object(path).await?;
    debug!("Fetched asset from storage [path={}, code={}]", path, code);

    if code != 200 {
        return Err(Error::msg(format!("Invalid response [code={}]", code)));
    }

    Ok(data)
}

// Maximum expiry time for pre-signed URLs in minio is 1 week.
pub async fn get_signed_url(bucket_name: &str, path: &str, expiry_secs: u32) -> Result<String> {
    let config = EntropyConfig::load()?.storage;
    let (creds, region) = get_creds(&config)?;
    let bucket = Bucket::new_with_path_style(bucket_name, region, creds)?;

    let url = bucket.presign_get(path, expiry_secs)?;
    debug!("Fetched pre-signed asset-url from storage [path={}, url={}]", path, url);

    Ok(url)
}
