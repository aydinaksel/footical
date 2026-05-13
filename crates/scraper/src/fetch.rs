use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;

const DELAY_BETWEEN_REQUESTS: Duration = Duration::from_millis(500);
const USER_AGENT: &str = "footical-scraper/0.1 (+https://footical.club)";

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client })
    }

    pub async fn fetch(&self, url: &str) -> anyhow::Result<String> {
        sleep(DELAY_BETWEEN_REQUESTS).await;
        let response = self.client.get(url).send().await?.error_for_status()?;
        let body = response.text().await?;
        Ok(body)
    }
}
