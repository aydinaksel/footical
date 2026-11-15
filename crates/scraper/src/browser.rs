use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use chromiumoxide::{Browser, BrowserConfig};
use futures_util::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;

const CHROMIUM_EXECUTABLE_VARIABLE: &str = "CHROMIUM_EXECUTABLE";
const XVFB_EXECUTABLE_VARIABLE: &str = "XVFB_EXECUTABLE";
const PROFILE_DIRECTORY_VARIABLE: &str = "CHROMIUM_PROFILE_DIRECTORY";
const DISPLAY_VARIABLE: &str = "DISPLAY";
const STANDARD_OUTPUT_DESCRIPTOR: &str = "1";
const WINDOW_WIDTH_PIXELS: u32 = 1734;
const WINDOW_HEIGHT_PIXELS: u32 = 1417;
const COLOUR_DEPTH_BITS: u32 = 24;
const DISPLAY_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const SETTLE_DURATION: Duration = Duration::from_secs(3);

pub struct BrowserOptions {
    chromium_executable: PathBuf,
    xvfb_executable: PathBuf,
    profile_directory: PathBuf,
}

impl BrowserOptions {
    pub fn from_environment() -> anyhow::Result<Self> {
        Ok(Self {
            chromium_executable: path_from(CHROMIUM_EXECUTABLE_VARIABLE)?,
            xvfb_executable: path_from(XVFB_EXECUTABLE_VARIABLE)?,
            profile_directory: path_from(PROFILE_DIRECTORY_VARIABLE)?,
        })
    }
}

fn path_from(variable: &str) -> anyhow::Result<PathBuf> {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{variable} is not set, so no browser can be launched"))
}

struct VirtualDisplay {
    name: String,
    _process: Child,
}

impl VirtualDisplay {
    async fn start(xvfb_executable: &Path) -> anyhow::Result<Self> {
        let mut process = Command::new(xvfb_executable)
            .arg("-displayfd")
            .arg(STANDARD_OUTPUT_DESCRIPTOR)
            .arg("-screen")
            .arg("0")
            .arg(format!(
                "{WINDOW_WIDTH_PIXELS}x{WINDOW_HEIGHT_PIXELS}x{COLOUR_DEPTH_BITS}"
            ))
            .arg("-nolisten")
            .arg("tcp")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("could not start Xvfb")?;
        let output = process
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Xvfb's output was not captured"))?;
        let mut line = String::new();
        tokio::time::timeout(
            DISPLAY_STARTUP_TIMEOUT,
            BufReader::new(output).read_line(&mut line),
        )
        .await
        .context("Xvfb did not report a display in time")?
        .context("could not read the display Xvfb chose")?;
        let number = line.trim();
        if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
            bail!("Xvfb reported {line:?} instead of a display number");
        }
        Ok(Self {
            name: format!(":{number}"),
            _process: process,
        })
    }
}

struct BrowserSession {
    browser: Browser,
    handler_task: JoinHandle<()>,
    _display: VirtualDisplay,
}

impl BrowserSession {
    async fn launch(options: &BrowserOptions) -> anyhow::Result<Self> {
        let display = VirtualDisplay::start(&options.xvfb_executable).await?;
        let configuration = BrowserConfig::builder()
            .chrome_executable(&options.chromium_executable)
            .with_head()
            .window_size(WINDOW_WIDTH_PIXELS, WINDOW_HEIGHT_PIXELS)
            .user_data_dir(&options.profile_directory)
            .request_timeout(REQUEST_TIMEOUT)
            .env(DISPLAY_VARIABLE, display.name.as_str())
            .arg("--ozone-platform=x11")
            .build()
            .map_err(|message| anyhow!("invalid Chromium configuration: {message}"))?;
        let (browser, mut handler) = Browser::launch(configuration)
            .await
            .context("could not launch Chromium")?;
        let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });
        Ok(Self {
            browser,
            handler_task,
            _display: display,
        })
    }

    async fn read(&self, url: &str) -> anyhow::Result<String> {
        let page = self.browser.new_page(url).await?;
        page.wait_for_navigation().await?;
        tokio::time::sleep(SETTLE_DURATION).await;
        let html = page.content().await?;
        page.close().await?;
        Ok(html)
    }

    async fn close(mut self) -> anyhow::Result<()> {
        self.browser.close().await?;
        self.handler_task.abort();
        Ok(())
    }
}

pub async fn read_in_fresh_browser(options: &BrowserOptions, url: &str) -> anyhow::Result<String> {
    let session = BrowserSession::launch(options).await?;
    let html = session.read(url).await;
    let closed = session.close().await;
    let html = html?;
    closed?;
    Ok(html)
}
