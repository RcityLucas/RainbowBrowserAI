use rainbow_shared::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_llm_service_basic_functionality() {
    let llm_service = MockLLMService::new();
    
    let context = Context {
        conversation_history: vec![],
        current_page_info: None,
        user_preferences: HashMap::new(),
        session_data: HashMap::new(),
    };

    // Test command parsing
    let parsed = llm_service
        .parse_command("go to stackoverflow and take screenshot", &context)
        .await
        .expect("Failed to parse command");
    
    assert_eq!(parsed.action, "screenshot");
    assert!(parsed.screenshot);
    assert_eq!(parsed.url.unwrap(), "https://stackoverflow.com");
    assert!(parsed.confidence > 0.8);

    // Test response generation
    let options = GenerationOptions::default();
    let response = llm_service
        .generate_response("Hello world", options)
        .await
        .expect("Failed to generate response");
    
    assert!(!response.content.is_empty());
    assert_eq!(response.provider, "mock");

    // Test usage stats
    let stats = llm_service
        .get_usage_stats()
        .await
        .expect("Failed to get stats");
    
    assert!(stats.total_requests > 0);
    assert!(stats.total_tokens > 0);
}

#[tokio::test]
async fn test_browser_service_config() {
    let config = BrowserPoolConfig::default();
    let _browser_service = WebDriverBrowserService::new(config.clone());
    
    // Test that the config has expected defaults
    assert_eq!(config.max_sessions, 10);
    assert_eq!(config.webdriver_url, "http://localhost:9515");
}

#[tokio::test]
async fn test_service_registry() {
    let browser_service: std::sync::Arc<dyn BrowserService> = std::sync::Arc::new(WebDriverBrowserService::new_default());
    let llm_service: std::sync::Arc<dyn LLMService> = std::sync::Arc::new(MockLLMService::new());
    
    let registry = ServiceRegistry::new(browser_service.clone(), llm_service.clone());
    
    // Test that services are properly registered
    assert!(std::sync::Arc::ptr_eq(&registry.browser, &browser_service));
    assert!(std::sync::Arc::ptr_eq(&registry.llm, &llm_service));
}

#[tokio::test]
async fn test_url_utilities() {
    // Test URL cleaning
    assert_eq!(clean_url("example.com"), "https://example.com");
    assert_eq!(clean_url("http://example.com/"), "http://example.com");
    assert_eq!(clean_url("HTTPS://EXAMPLE.COM"), "https://example.com");
    
    // Test URL validation
    assert!(is_valid_url("https://example.com"));
    assert!(is_valid_url("http://localhost:8080"));
    assert!(!is_valid_url("not-a-url"));
    
    // Test filename generation
    let filename = url_to_filename("https://example.com/path?query=1");
    assert_eq!(filename, "https___example.com_path_query=1");
}

#[tokio::test]
async fn test_cost_calculation() {
    // Test OpenAI cost calculation
    let cost = cost::calculate_openai_cost(1000, 500, "gpt-3.5-turbo");
    assert!((cost - 0.001250).abs() < 0.000001);
    
    // Test Claude cost calculation
    let cost = cost::calculate_claude_cost(1000, 500, "claude-3-haiku");
    assert!((cost - 0.000875).abs() < 0.000001);
}