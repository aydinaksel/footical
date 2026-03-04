use crate::config::R2Config;
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;

pub async fn upload_all(r2: &R2Config, output_directory: &Path) -> anyhow::Result<()> {
    let client = build_client(r2);

    for filename in ["leagues.json", "divisions.json", "teams.json"] {
        let path = output_directory.join(filename);
        upload_file(&client, &r2.bucket, &path, filename, "application/json").await?;
        println!("  uploaded {}", filename);
    }

    for entry in std::fs::read_dir(output_directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("ics") {
            let key = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow::anyhow!("non-UTF-8 filename in output directory"))?
                .to_string();
            upload_file(&client, &r2.bucket, &path, &key, "text/calendar; charset=utf-8").await?;
            println!("  uploaded {}", key);
        }
    }

    Ok(())
}

fn build_client(r2: &R2Config) -> aws_sdk_s3::Client {
    use aws_sdk_s3::config::{Builder, Credentials, Region};

    let credentials = Credentials::new(
        &r2.access_key_id,
        &r2.secret_access_key,
        None,
        None,
        "footical-generator",
    );

    let config = Builder::new()
        .credentials_provider(credentials)
        .endpoint_url(&r2.endpoint_url)
        .region(Region::new("auto"))
        .force_path_style(true)
        .behavior_version_latest()
        .build();

    aws_sdk_s3::Client::from_conf(config)
}

async fn upload_file(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    path: &Path,
    key: &str,
    content_type: &str,
) -> anyhow::Result<()> {
    let content = std::fs::read(path)?;
    let body = ByteStream::from(content);

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .body(body)
        .send()
        .await?;

    Ok(())
}
