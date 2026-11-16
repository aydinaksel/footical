use leptos::prelude::*;
use leptos::server_fn::codec::Json;

use crate::types::{FulltimePageLink, SavedSourcePage, StoredSourcePage};

#[cfg(feature = "ssr")]
use crate::server::auth::require_admin;
#[cfg(feature = "ssr")]
use crate::server::database::{database_pool, report_query_failure};

#[server]
pub async fn list_source_pages() -> Result<Vec<StoredSourcePage>, ServerFnError> {
    require_admin().await?;
    let pool = database_pool()?;
    sqlx::query_as::<_, StoredSourcePage>(
        "SELECT source_page.source_page_id,
                source_page.page_kind,
                source_page.page_key,
                length(CAST(source_page.body AS BLOB)) AS size_bytes,
                datetime(source_page.fetched_at, 'localtime') AS fetched_at
         FROM source_page
         JOIN source ON source.source_id = source_page.source_id
         WHERE source.code = 'fa-fulltime'
         ORDER BY source_page.fetched_at DESC, source_page.page_kind",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_source_pages", error))
}

#[server]
pub async fn list_fulltime_page_links() -> Result<Vec<FulltimePageLink>, ServerFnError> {
    require_admin().await?;
    Ok(footical_scraper::manual_page_links()
        .into_iter()
        .map(|link| FulltimePageLink {
            page_kind: link.page_kind,
            url: link.url,
        })
        .collect())
}

#[server(input = Json)]
pub async fn save_source_page(html: String) -> Result<SavedSourcePage, ServerFnError> {
    require_admin().await?;
    let pool = database_pool()?;
    let pasted = footical_scraper::store_pasted_page(&pool, &html)
        .await
        .map_err(|error| ServerFnError::new(format!("{error:#}")))?;
    Ok(SavedSourcePage {
        page_kind: pasted.page_kind,
        fixtures_parsed: pasted.fixtures_parsed,
        players_parsed: pasted.players_parsed,
        issues: pasted.issues,
    })
}
