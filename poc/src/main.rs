use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use rainbow_poc::{
    Config, SimpleBrowser,
    llm_service::LLMService,
    ConversationContext, ExecutionResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

#[derive(Parser)]
#[command(name = "rainbow-poc")]
#[command(about = "RainbowBrowserAI POC - AI-powered browser automation")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    
    /// Test browser connectivity
    Test,
    
    /// Execute a natural language command
    Ask {
        /// The command to execute
        command: String,
    },
    
    /// Navigate to a URL
    Navigate {
        /// The URL to navigate to
        url: String,
        
        /// Take a screenshot after navigation
        #[arg(short, long)]
        screenshot: bool,
    },
}

#[derive(Clone)]
struct AppState {
    browser: Arc<Mutex<SimpleBrowser>>,
    llm_service: Arc<LLMService>,
    context: Arc<Mutex<ConversationContext>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::load(None::<&str>)?;
    info!("Configuration loaded successfully");
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Serve { port }) => {
            serve_api(port, config).await?;
        }
        Some(Commands::Test) => {
            test_browser().await?;
        }
        Some(Commands::Ask { command }) => {
            execute_command(&command, config).await?;
        }
        Some(Commands::Navigate { url, screenshot }) => {
            navigate_to_url(&url, screenshot).await?;
        }
        None => {
            // Default to serving if no command provided
            serve_api(3000, config).await?;
        }
    }
    
    Ok(())
}

async fn serve_api(port: u16, mut config: Config) -> anyhow::Result<()> {
    // Update the config with the specified port
    // The API module will use config.api.port, so we need to ensure it exists
    if config.api.is_none() {
        config.api = Some(rainbow_poc::config::ApiConfig {
            enabled: true,
            port: Some(port),
            auth_token: None,
        });
    } else {
        config.api.as_mut().unwrap().port = Some(port);
    }
    
    // Use the comprehensive API module instead of the simple routes
    rainbow_poc::api::start_server(config).await
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[derive(Deserialize)]
struct AskRequest {
    command: String,
}

#[derive(Serialize)]
struct AskResponse {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

async fn ask_endpoint(
    State(state): State<AppState>,
    Json(request): Json<AskRequest>,
) -> impl IntoResponse {
    match execute_with_state(&request.command, state).await {
        Ok(result) => {
            (StatusCode::OK, Json(AskResponse {
                success: true,
                result: Some(result),
                error: None,
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AskResponse {
                success: false,
                result: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

#[derive(Deserialize)]
struct NavigateRequest {
    url: String,
    screenshot: Option<bool>,
}

async fn navigate_endpoint(
    State(state): State<AppState>,
    Json(request): Json<NavigateRequest>,
) -> impl IntoResponse {
    let browser = state.browser.lock().await;
    
    match browser.navigate_to(&request.url).await {
        Ok(_) => {
            let mut response = serde_json::json!({
                "success": true,
                "url": request.url
            });
            
            if request.screenshot.unwrap_or(false) {
                if let Ok(_) = browser.take_screenshot("api_screenshot.png").await {
                    response["screenshot"] = serde_json::Value::String("api_screenshot.png".to_string());
                }
            }
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })))
        }
    }
}

async fn test_browser() -> anyhow::Result<()> {
    info!("Testing browser connectivity...");
    
    let browser = SimpleBrowser::new().await?;
    browser.navigate_to("https://example.com").await?;
    
    let title = browser.get_title().await?;
    info!("Page title: {}", title);
    
    info!("✅ Browser test successful!");
    Ok(())
}

async fn execute_command(command: &str, config: Config) -> anyhow::Result<()> {
    info!("Executing command: {}", command);
    
    let browser = Arc::new(Mutex::new(SimpleBrowser::new().await?));
    let llm_service = Arc::new(LLMService::new(config.llm.api_key.clone().unwrap_or_default()));
    let context = Arc::new(Mutex::new(ConversationContext::new()));
    
    let state = AppState {
        browser,
        llm_service,
        context,
    };
    
    let result = execute_with_state(command, state).await?;
    println!("{}", result);
    
    Ok(())
}

async fn execute_with_state(command: &str, state: AppState) -> anyhow::Result<String> {
    // Parse the command using LLM
    use rainbow_poc::CostTracker;
    let mut cost_tracker = CostTracker::new(100.0);
    let parsed = state.llm_service.parse_natural_command(command, &mut cost_tracker).await?;
    
    let mut browser = state.browser.lock().await;
    let mut context = state.context.lock().await;
    
    // Execute based on parsed command
    let result = match parsed.action.as_str() {
        "navigate" => {
            if let Some(url) = &parsed.url {
                browser.navigate_to(url).await?;
                format!("Navigated to {}", url)
            } else {
                "Missing URL parameter".to_string()
            }
        }
        "screenshot" => {
            let filename = parsed.filename.as_deref().unwrap_or("command_screenshot.png");
            browser.take_screenshot(filename).await?;
            format!("Screenshot saved: {}", filename)
        }
        _ => {
            // For other actions, try to navigate to the URL if provided
            if let Some(url) = &parsed.url {
                browser.navigate_to(url).await?;
                if parsed.screenshot {
                    let filename = parsed.filename.as_deref().unwrap_or("screenshot.png");
                    browser.take_screenshot(filename).await?;
                    format!("Navigated to {} and took screenshot", url)
                } else {
                    format!("Navigated to {}", url)
                }
            } else {
                format!("Action '{}' not fully implemented", parsed.action)
            }
        }
    };
    
    // Record in context
    use rainbow_poc::HistoryEntry;
    context.add_history_entry(HistoryEntry {
        timestamp: chrono::Utc::now(),
        user_input: command.to_string(),
        parsed_command: parsed.clone(),
        execution_result: ExecutionResult {
            success: true,
            duration_ms: 1000,
            error_message: None,
            output_summary: result.clone(),
        },
        cost: 0.0,
    }).ok();
    
    Ok(result)
}

async fn navigate_to_url(url: &str, screenshot: bool) -> anyhow::Result<()> {
    info!("Navigating to {}", url);
    
    let browser = SimpleBrowser::new().await?;
    browser.navigate_to(url).await?;
    
    if screenshot {
        browser.take_screenshot("screenshot.png").await?;
        info!("Screenshot saved to screenshot.png");
    }
    
    info!("✅ Navigation successful!");
    Ok(())
}