use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub async fn store(
    pool: &SqlitePool,
    source_id: i32,
    page_kind: &str,
    page_key: &str,
    body: &str,
) -> anyhow::Result<()> {
    let content_hash = format!("{:x}", Sha256::digest(body.as_bytes()));
    sqlx::query(
        "INSERT INTO source_page (source_id, page_kind, page_key, content_hash, body)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (source_id, page_kind, page_key) DO UPDATE SET
           content_hash = excluded.content_hash,
           body = excluded.body,
           fetched_at = CURRENT_TIMESTAMP",
    )
    .bind(source_id)
    .bind(page_kind)
    .bind(page_key)
    .bind(content_hash)
    .bind(body)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FA_FULLTIME_SOURCE_ID: i32 = 2;

    async fn migrated_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../website/migrations")
            .run(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn storing_a_page_again_replaces_its_body() {
        let pool = migrated_pool().await;
        store(
            &pool,
            FA_FULLTIME_SOURCE_ID,
            "table",
            "1_780926466",
            "first",
        )
        .await
        .unwrap();
        store(
            &pool,
            FA_FULLTIME_SOURCE_ID,
            "table",
            "1_780926466",
            "second",
        )
        .await
        .unwrap();
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT body, content_hash FROM source_page")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(
            rows,
            vec![(
                "second".to_owned(),
                "16367aacb67a4a017c8da8ab95682ccb390863780f7114dda0a0e0c55644c7c4".to_owned()
            )]
        );
    }
}
