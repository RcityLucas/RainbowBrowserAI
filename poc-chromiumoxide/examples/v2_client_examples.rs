// Example client code for RainbowBrowserAI V2 Coordinated API
// This file demonstrates how to use the new coordinated endpoints

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

// API Response structures
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CoordinatedResponse<T> {
    success: bool,
    session_id: String,
    data: Option<T>,
    error: Option<String>,
    metrics: Option<OperationMetrics>,
}

#[derive(Debug, Deserialize)]
struct OperationMetrics {
    duration_ms: u64,
    cache_hits: u32,
    cache_misses: u32,
}

#[derive(Debug, Deserialize)]
struct SessionInfo {
    session_id: String,
    created: bool,
    modules: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct NavigationResult {
    url: String,
    load_time_ms: u64,
    success: bool,
    analysis: Option<PageAnalysis>,
}

#[derive(Debug, Deserialize)]
struct PageAnalysis {
    title: String,
    element_count: usize,
    interactive_elements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct IntelligentActionResult {
    success: bool,
    duration_ms: u64,
    analysis: ActionAnalysis,
    plan: ActionPlan,
    execution: ExecutionResult,
    verification: VerificationResult,
    learning_applied: bool,
}

#[derive(Debug, Deserialize)]
struct ActionAnalysis {
    elements_found: usize,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct ActionPlan {
    steps: Vec<String>,
    tools_required: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExecutionResult {
    success: bool,
    output: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct VerificationResult {
    success: bool,
    confidence: f64,
    error: Option<String>,
}

// Client wrapper for V2 API
struct V2ApiClient {
    client: Client,
    base_url: String,
    session_id: Option<String>,
}

impl V2ApiClient {
    fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            session_id: None,
        }
    }

    // Create a new coordinated session
    async fn create_session(&mut self) -> Result<String> {
        let url = format!("{}/api/v2/session/create", self.base_url);
        let response = self.client.post(&url).send().await?;

        let api_response: ApiResponse<SessionInfo> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                self.session_id = Some(data.session_id.clone());
                println!("✅ Created session: {}", data.session_id);
                println!("   Modules initialized: {:?}", data.modules);
                return Ok(data.session_id);
            }
        }

        Err(anyhow::anyhow!(
            "Failed to create session: {:?}",
            api_response.error
        ))
    }

    // Navigate to a URL with optional page analysis
    async fn navigate(&self, url: &str, analyze: bool) -> Result<NavigationResult> {
        let api_url = format!("{}/api/v2/navigate", self.base_url);
        let request = json!({
            "session_id": self.session_id,
            "data": {
                "url": url,
                "wait_for_load": true,
                "analyze_page": analyze
            }
        });

        let response = self.client.post(&api_url).json(&request).send().await?;

        let api_response: CoordinatedResponse<NavigationResult> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                println!("✅ Navigated to: {}", data.url);
                println!("   Load time: {}ms", data.load_time_ms);
                if let Some(metrics) = api_response.metrics {
                    println!("   Operation took: {}ms", metrics.duration_ms);
                    println!(
                        "   Cache hits: {}, misses: {}",
                        metrics.cache_hits, metrics.cache_misses
                    );
                }
                if let Some(ref analysis) = data.analysis {
                    println!("   Page title: {}", analysis.title);
                    println!("   Elements: {}", analysis.element_count);
                    println!(
                        "   Interactive: {} elements",
                        analysis.interactive_elements.len()
                    );
                }
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!(
            "Navigation failed: {:?}",
            api_response.error
        ))
    }

    // Execute an intelligent action
    async fn intelligent_action(
        &self,
        action_type: &str,
        target: &str,
    ) -> Result<IntelligentActionResult> {
        let url = format!("{}/api/v2/intelligent-action", self.base_url);
        let request = json!({
            "session_id": self.session_id,
            "data": {
                "action_type": action_type,
                "target": target,
                "parameters": {}
            }
        });

        let response = self.client.post(&url).json(&request).send().await?;

        let api_response: CoordinatedResponse<IntelligentActionResult> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                println!("✅ Intelligent action completed: {}", action_type);
                println!("   Target: {}", target);
                println!("   Success: {}", data.success);
                println!("   Duration: {}ms", data.duration_ms);
                println!("   Confidence: {:.2}%", data.analysis.confidence * 100.0);
                println!("   Plan steps: {}", data.plan.steps.len());
                println!("   Verification: {:?}", data.verification.success);
                println!("   Learning applied: {}", data.learning_applied);
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!(
            "Intelligent action failed: {:?}",
            api_response.error
        ))
    }

    // Analyze current page with perception
    async fn analyze_page(&self, analysis_type: Option<&str>) -> Result<PageAnalysis> {
        let url = format!("{}/api/v2/perception/analyze", self.base_url);
        let request = json!({
            "session_id": self.session_id,
            "data": {
                "analysis_type": analysis_type.unwrap_or("standard"),
                "target": "current_page"
            }
        });

        let response = self.client.post(&url).json(&request).send().await?;

        let api_response: CoordinatedResponse<PageAnalysis> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                println!("✅ Page analysis complete");
                println!("   Title: {}", data.title);
                println!("   Total elements: {}", data.element_count);
                println!(
                    "   Interactive elements: {}",
                    data.interactive_elements.len()
                );
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!(
            "Page analysis failed: {:?}",
            api_response.error
        ))
    }

    // Execute a tool
    async fn execute_tool(
        &self,
        tool_name: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v2/tool/execute", self.base_url);
        let request = json!({
            "session_id": self.session_id,
            "data": {
                "tool_name": tool_name,
                "parameters": parameters
            }
        });

        let response = self.client.post(&url).json(&request).send().await?;

        let api_response: CoordinatedResponse<serde_json::Value> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                println!("✅ Tool executed: {}", tool_name);
                if let Some(metrics) = api_response.metrics {
                    println!("   Duration: {}ms", metrics.duration_ms);
                }
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!(
            "Tool execution failed: {:?}",
            api_response.error
        ))
    }

    // Get session health
    async fn get_session_health(&self) -> Result<serde_json::Value> {
        let session_id = self
            .session_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let url = format!("{}/api/v2/session/{}", self.base_url, session_id);
        let response = self.client.get(&url).send().await?;

        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                println!("✅ Session health retrieved");
                println!("   {}", serde_json::to_string_pretty(&data)?);
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!(
            "Failed to get session health: {:?}",
            api_response.error
        ))
    }

    // Get system health
    async fn get_system_health(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/v2/health", self.base_url);
        let response = self.client.get(&url).send().await?;

        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        if api_response.success {
            if let Some(data) = api_response.data {
                println!("✅ System health:");
                println!("   {}", serde_json::to_string_pretty(&data)?);
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!(
            "Failed to get system health: {:?}",
            api_response.error
        ))
    }

    // Delete session
    async fn delete_session(&mut self) -> Result<()> {
        if let Some(session_id) = &self.session_id {
            let url = format!("{}/api/v2/session/{}", self.base_url, session_id);
            let response = self.client.delete(&url).send().await?;

            let api_response: ApiResponse<serde_json::Value> = response.json().await?;
            if api_response.success {
                println!("✅ Session deleted: {}", session_id);
                self.session_id = None;
                return Ok(());
            }

            return Err(anyhow::anyhow!(
                "Failed to delete session: {:?}",
                api_response.error
            ));
        }

        Ok(())
    }
}

// Example 1: Basic navigation and analysis
async fn example_basic_navigation() -> Result<()> {
    println!("\n🚀 Example 1: Basic Navigation and Analysis");
    println!("=".repeat(50));

    let mut client = V2ApiClient::new("http://localhost:3000");

    // Create a session
    client.create_session().await?;

    // Navigate to a website with analysis
    client.navigate("https://example.com", true).await?;

    // Wait a bit
    sleep(Duration::from_secs(1)).await;

    // Analyze the page
    client.analyze_page(Some("standard")).await?;

    // Clean up
    client.delete_session().await?;

    Ok(())
}

// Example 2: Intelligent actions workflow
async fn example_intelligent_actions() -> Result<()> {
    println!("\n🤖 Example 2: Intelligent Actions Workflow");
    println!("=".repeat(50));

    let mut client = V2ApiClient::new("http://localhost:3000");

    // Create a session
    client.create_session().await?;

    // Navigate to a form page
    client.navigate("https://example.com/form", true).await?;

    // Fill in a form field intelligently
    client
        .intelligent_action("type", "email input field")
        .await?;

    // Click submit button intelligently
    client.intelligent_action("click", "submit button").await?;

    // Wait for navigation
    sleep(Duration::from_secs(2)).await;

    // Analyze result page
    client.analyze_page(Some("deep")).await?;

    // Clean up
    client.delete_session().await?;

    Ok(())
}

// Example 3: Tool execution with coordination
async fn example_tool_execution() -> Result<()> {
    println!("\n🔧 Example 3: Coordinated Tool Execution");
    println!("=".repeat(50));

    let mut client = V2ApiClient::new("http://localhost:3000");

    // Create a session
    client.create_session().await?;

    // Navigate
    client.navigate("https://example.com", false).await?;

    // Execute click tool
    let click_params = json!({
        "selector": "#main-button"
    });
    client.execute_tool("click", click_params).await?;

    // Execute extract_text tool
    let extract_params = json!({
        "selector": "h1"
    });
    let text = client.execute_tool("extract_text", extract_params).await?;
    println!("   Extracted text: {}", text);

    // Execute screenshot tool
    let screenshot_params = json!({
        "full_page": false
    });
    client
        .execute_tool("take_screenshot", screenshot_params)
        .await?;

    // Clean up
    client.delete_session().await?;

    Ok(())
}

// Example 4: Health monitoring
async fn example_health_monitoring() -> Result<()> {
    println!("\n💚 Example 4: Health Monitoring");
    println!("=".repeat(50));

    let mut client = V2ApiClient::new("http://localhost:3000");

    // Check system health before creating session
    client.get_system_health().await?;

    // Create a session
    client.create_session().await?;

    // Perform some operations
    client.navigate("https://example.com", true).await?;

    // Check session health
    client.get_session_health().await?;

    // More operations
    client
        .intelligent_action("click", "navigation link")
        .await?;

    // Check health again
    client.get_session_health().await?;

    // Check system health with active session
    client.get_system_health().await?;

    // Clean up
    client.delete_session().await?;

    Ok(())
}

// Example 5: Multi-session coordination
async fn example_multi_session() -> Result<()> {
    println!("\n👥 Example 5: Multi-Session Coordination");
    println!("=".repeat(50));

    // Create multiple clients/sessions
    let mut client1 = V2ApiClient::new("http://localhost:3000");
    let mut client2 = V2ApiClient::new("http://localhost:3000");

    // Create sessions
    let session1 = client1.create_session().await?;
    println!("Session 1: {}", session1);

    let session2 = client2.create_session().await?;
    println!("Session 2: {}", session2);

    // Parallel operations
    let (result1, result2) = tokio::join!(
        client1.navigate("https://example.com", true),
        client2.navigate("https://google.com", true)
    );

    result1?;
    result2?;

    // Check system health with multiple sessions
    client1.get_system_health().await?;

    // Clean up both sessions
    client1.delete_session().await?;
    client2.delete_session().await?;

    Ok(())
}

// Example 6: Error handling and recovery
async fn example_error_handling() -> Result<()> {
    println!("\n⚠️ Example 6: Error Handling and Recovery");
    println!("=".repeat(50));

    let mut client = V2ApiClient::new("http://localhost:3000");

    // Create a session
    client.create_session().await?;

    // Try to navigate to an invalid URL
    match client.navigate("not-a-valid-url", false).await {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }

    // Session should still be valid, try a valid operation
    client.navigate("https://example.com", false).await?;

    // Try an intelligent action that might fail
    match client
        .intelligent_action("click", "non-existent-element")
        .await
    {
        Ok(result) => {
            if !result.success {
                println!("Action failed as expected");
                if let Some(error) = result.verification.error {
                    println!("Verification error: {}", error);
                }
            }
        }
        Err(e) => println!("Request error: {}", e),
    }

    // Session should still be healthy
    client.get_session_health().await?;

    // Clean up
    client.delete_session().await?;

    Ok(())
}

// Main function to run all examples
#[tokio::main]
async fn main() -> Result<()> {
    println!("🌈 RainbowBrowserAI V2 API Client Examples");
    println!("=".repeat(60));

    // Run examples sequentially
    example_basic_navigation().await?;
    example_intelligent_actions().await?;
    example_tool_execution().await?;
    example_health_monitoring().await?;
    example_multi_session().await?;
    example_error_handling().await?;

    println!("\n✅ All examples completed successfully!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = V2ApiClient::new("http://localhost:3000");
        assert!(client.session_id.is_none());
    }

    #[tokio::test]
    async fn test_session_lifecycle() -> Result<()> {
        let mut client = V2ApiClient::new("http://localhost:3000");

        // Create session
        let session_id = client.create_session().await?;
        assert!(!session_id.is_empty());
        assert_eq!(client.session_id, Some(session_id.clone()));

        // Delete session
        client.delete_session().await?;
        assert!(client.session_id.is_none());

        Ok(())
    }
}
