use anyhow::Result;
use clap::{Parser, Subcommand};
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod browser;
mod coordination;
mod intelligence;
mod llm;
mod perception;
mod tools;

use browser::Browser;

#[derive(Parser)]
#[command(name = "rainbow-poc-chromiumoxide")]
#[command(about = "RainbowBrowserAI with chromiumoxide", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3001")]
        port: u16,

        /// Enable headless mode
        #[arg(long)]
        headless: bool,
    },

    /// Navigate to a URL and take a screenshot
    Navigate {
        /// URL to navigate to
        url: String,

        /// Save screenshot to file
        #[arg(short, long)]
        screenshot: Option<String>,

        /// Enable headless mode
        #[arg(long)]
        headless: bool,
    },

    /// Execute a browser automation workflow
    Workflow {
        /// Path to workflow YAML file
        file: String,

        /// Enable headless mode
        #[arg(long)]
        headless: bool,
    },

    /// Test browser connection
    Test {
        /// Enable headless mode
        #[arg(long)]
        headless: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Load environment variables
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port, headless } => {
            serve_api(port, headless).await?;
        }
        Commands::Navigate {
            url,
            screenshot,
            headless,
        } => {
            navigate_and_screenshot(&url, screenshot, headless).await?;
        }
        Commands::Workflow { file, headless } => {
            execute_workflow(&file, headless).await?;
        }
        Commands::Test { headless } => {
            test_browser(headless).await?;
        }
    }

    Ok(())
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rainbow_poc_chromiumoxide=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn serve_api(port: u16, headless: bool) -> Result<()> {
    info!("Starting API server on port {}", port);
    info!(
        "Browser mode: {}",
        if headless { "headless" } else { "headed" }
    );

    // Initialize browser pool with headless mode (3 browsers max to prevent excessive windows)
    // Do not preload browsers at startup so the API can come up even if
    // Chromium/headless deps are not available yet. Browsers will be created lazily.
    let pool = browser::pool::BrowserPool::new_with_headless(3, headless)?;

    // Start API server
    api::serve(port, pool).await?;

    Ok(())
}

async fn navigate_and_screenshot(
    url: &str,
    screenshot: Option<String>,
    headless: bool,
) -> Result<()> {
    info!("Navigating to: {}", url);

    let browser = if headless {
        Browser::new_headless().await?
    } else {
        Browser::new_headed().await?
    };

    // Navigate to URL
    browser.navigate_to(url).await?;

    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Take screenshot if requested
    if let Some(path) = screenshot {
        info!("Taking screenshot: {}", path);
        let screenshot_data = browser
            .screenshot(browser::ScreenshotOptions::default())
            .await?;
        std::fs::write(&path, screenshot_data)?;
        info!("Screenshot saved to: {}", path);
    }

    // Get current URL
    let current_url = browser.current_url().await?;
    info!("Current URL: {}", current_url);

    browser.close().await?;
    info!("Browser closed");

    Ok(())
}

async fn execute_workflow(file: &str, headless: bool) -> Result<()> {
    info!("Executing workflow from: {}", file);

    // Read workflow file
    let workflow_content = std::fs::read_to_string(file)?;
    let workflow: serde_yaml::Value = serde_yaml::from_str(&workflow_content)?;

    info!("Workflow loaded: {:?}", workflow);

    // TODO: Implement workflow execution
    error!("Workflow execution not yet implemented");

    Ok(())
}

async fn test_browser(headless: bool) -> Result<()> {
    info!("Testing browser connection...");

    let browser = if headless {
        info!("Creating headless browser...");
        Browser::new_headless().await?
    } else {
        info!("Creating headed browser...");
        Browser::new_headed().await?
    };

    info!("Browser created successfully");

    // Test navigation
    info!("Testing navigation to example.com...");
    browser.navigate_to("https://example.com").await?;
    info!("Navigation successful");

    // Test getting current URL
    let url = browser.current_url().await?;
    info!("Current URL: {}", url);

    // Test finding element
    info!("Testing element finding...");
    let element = browser.find_element("h1").await?;
    info!("Found element: {:?}", element);

    // Test getting text
    let text = browser.get_text("h1").await?;
    info!("H1 text: {}", text);

    // Test screenshot
    info!("Testing screenshot...");
    let screenshot = browser
        .screenshot(browser::ScreenshotOptions::default())
        .await?;
    info!("Screenshot taken, size: {} bytes", screenshot.len());

    // Test JavaScript execution
    info!("Testing JavaScript execution...");
    let result = browser.execute_script("return document.title").await?;
    info!("Page title: {:?}", result);

    browser.close().await?;
    info!("All tests passed successfully!");

    Ok(())
}
