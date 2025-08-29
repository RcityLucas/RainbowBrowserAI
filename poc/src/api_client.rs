//! API client for calling the RainbowBrowserAI server
//! This allows the CLI to communicate with a running server instance
//! instead of directly connecting to ChromeDriver

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use std::time::Duration;

/// API client for communicating with the RainbowBrowserAI server
pub struct ApiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct CommandRequest {
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub action: String,
    pub confidence: f32,
    pub result: Option<serde_json::Value>,
    pub explanation: String,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url,
        }
    }
    
    /// Check if the server is available
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        debug!("Checking server health at: {}", url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Server is healthy at {}", self.base_url);
                    Ok(true)
                } else {
                    warn!("Server returned non-success status: {}", response.status());
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Server health check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Execute a natural language command via the API
    pub async fn execute_command(&self, command: &str) -> Result<CommandResponse> {
        let url = format!("{}/command", self.base_url);
        info!("Sending command to API: {}", command);
        
        let request = CommandRequest {
            command: command.to_string(),
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send command to API")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("API returned error status {}: {}", status, error_text);
            anyhow::bail!("API error: {} - {}", status, error_text);
        }
        
        let result: CommandResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;
        
        debug!("API response: {:?}", result);
        
        Ok(result)
    }
    
    /// Execute a workflow via the API
    pub async fn execute_workflow(&self, workflow_steps: Vec<String>) -> Result<serde_json::Value> {
        let url = format!("{}/workflow", self.base_url);
        info!("Sending workflow to API with {} steps", workflow_steps.len());
        
        let request = serde_json::json!({
            "steps": workflow_steps
        });
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send workflow to API")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("API returned error status {}: {}", status, error_text);
            anyhow::bail!("API error: {} - {}", status, error_text);
        }
        
        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse API response")?;
        
        debug!("Workflow API response: {:?}", result);
        
        Ok(result)
    }
}

/// Try to use the API client, fall back to direct browser if API is not available
pub async fn execute_via_api_or_direct(
    command: &str,
    api_url: Option<String>,
) -> Result<String> {
    // Default to localhost:3001 if no URL provided
    let api_url = api_url.unwrap_or_else(|| "http://localhost:3001".to_string());
    
    // Try API first
    let api_client = ApiClient::new(api_url.clone());
    
    // Check if server is available
    if api_client.health_check().await.unwrap_or(false) {
        info!("Using API server at {}", api_url);
        
        match api_client.execute_command(command).await {
            Ok(response) => {
                if response.success {
                    Ok(format!(
                        "✅ Success: {}\nAction: {}\nConfidence: {:.1}%\nExplanation: {}",
                        response.action,
                        response.action,
                        response.confidence * 100.0,
                        response.explanation
                    ))
                } else {
                    Ok(format!(
                        "❌ Failed: {}\nExplanation: {}",
                        response.action,
                        response.explanation
                    ))
                }
            }
            Err(e) => {
                warn!("API call failed: {}", e);
                anyhow::bail!("API call failed: {}", e)
            }
        }
    } else {
        // API not available, return error suggesting to start the server
        anyhow::bail!(
            "Cannot connect to API server at {}. Please start the server with:\n\
             cargo run -- serve --port 3001\n\
             Or set RAINBOW_API_URL environment variable to the correct server URL",
            api_url
        )
    }
}