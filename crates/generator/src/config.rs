use std::path::PathBuf;

pub struct Config {
    pub database_url: String,
    pub output_directory: PathBuf,
    pub r2: Option<R2Config>,
}

pub struct R2Config {
    pub endpoint_url: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

pub fn load() -> anyhow::Result<Config> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("FOOTICAL_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .map_err(|_| anyhow::anyhow!("neither FOOTICAL_DATABASE_URL nor DATABASE_URL is set"))?;

    let output_directory = std::env::var("OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("output"));

    let r2 = load_r2_config()?;

    Ok(Config { database_url, output_directory, r2 })
}

fn load_r2_config() -> anyhow::Result<Option<R2Config>> {
    let account_id = std::env::var("R2_ACCOUNT_ID").ok();
    let bucket = std::env::var("R2_BUCKET").ok();
    let access_key_id = std::env::var("R2_ACCESS_KEY_ID").ok();
    let secret_access_key = std::env::var("R2_SECRET_ACCESS_KEY").ok();

    match (account_id, bucket, access_key_id, secret_access_key) {
        (None, None, None, None) => Ok(None),
        (Some(account_id), Some(bucket), Some(access_key_id), Some(secret_access_key)) => {
            Ok(Some(R2Config {
                endpoint_url: format!("https://{}.r2.cloudflarestorage.com", account_id),
                bucket,
                access_key_id,
                secret_access_key,
            }))
        }
        _ => Err(anyhow::anyhow!(
            "R2 is partially configured — set all of R2_ACCOUNT_ID, R2_BUCKET, \
             R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY or none of them"
        )),
    }
}
