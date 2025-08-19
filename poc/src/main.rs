use clap::{Parser, Subcommand};
use rainbow_poc::{SimpleBrowser, CostTracker, Config, ScreenshotOptions, LLMService, ConversationContext, HistoryEntry, ExecutionResult, Workflow, WorkflowEngine, start_server, DataExtractor, TaskExecutor, TaskUnderstanding, MockTaskUnderstanding};
use anyhow::{Result, Context};
use tracing::{info, error, warn};
use chrono::Utc;
use std::time::Duration;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "rainbow-poc")]
#[command(about = "RainbowBrowserAI Proof of Concept - Simple browser automation")]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Navigate to a single URL
    Navigate {
        /// The URL to navigate to
        url: String,

        /// Take a screenshot after navigation
        #[arg(short, long)]
        screenshot: bool,

        /// Custom screenshot filename (default: timestamp)
        #[arg(long)]
        filename: Option<String>,

        /// Use viewport screenshot instead of full page
        #[arg(long)]
        viewport_only: bool,

        /// Custom viewport width
        #[arg(long, default_value = "1920")]
        width: u32,

        /// Custom viewport height
        #[arg(long, default_value = "1080")]
        height: u32,
    },

    /// Test multiple websites for reliability
    Test {
        /// URLs to test (comma-separated or multiple --url flags)
        #[arg(short, long, value_delimiter = ',')]
        urls: Vec<String>,

        /// Take screenshots of all sites
        #[arg(short, long)]
        screenshots: bool,

        /// Number of retry attempts per site
        #[arg(long, default_value = "3")]
        retries: u32,

        /// Timeout per site in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// Show cost report
    Report,

    /// Parse and execute natural language commands
    Ask {
        /// Natural language command (e.g., "navigate to google and take a screenshot")
        command: String,
    },

    /// Execute a workflow from YAML or JSON file
    Workflow {
        /// Path to the workflow file (YAML or JSON)
        file: String,
        
        /// Input variables as key=value pairs
        #[arg(short, long, value_delimiter = ',')]
        inputs: Vec<String>,
        
        /// Dry run - validate workflow without execution
        #[arg(long)]
        dry_run: bool,
    },

    /// Start the REST API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Enable API documentation UI
        #[arg(long)]
        docs: bool,
    },
    
    /// Extract data from a webpage
    Extract {
        /// URL to extract data from
        url: String,
        
        /// Output format (json, csv, or text)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// CSS selector for specific extraction
        #[arg(short, long)]
        selector: Option<String>,
        
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    if let Err(e) = dotenv::dotenv() {
        // .env file is optional, so only warn if it exists but can't be read
        if !e.to_string().contains("not found") {
            eprintln!("Warning: Could not load .env file: {}", e);
        }
    }

    // Initialize tracing
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    info!("🌈 RainbowBrowserAI PoC Starting...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded - Daily budget: ${:.2}", config.daily_budget());

    // Load or create cost tracker
    let mut cost_tracker = CostTracker::load_from_file()
        .unwrap_or_else(|_| {
            info!("Creating new cost tracker");
            CostTracker::new(config.daily_budget())
        });

    // Handle commands
    let result = match args.command {
        Commands::Navigate { 
            url, screenshot, filename, viewport_only, width, height 
        } => {
            let estimated_cost = cost_tracker.estimate_browser_operation_cost();
            if !cost_tracker.can_afford(estimated_cost) {
                error!("❌ Daily budget exceeded! Cannot proceed.");
                println!("\n{}", cost_tracker.generate_daily_report());
                return Ok(());
            }

            execute_navigation(
                &url, screenshot, filename, viewport_only, width, height, &mut cost_tracker
            ).await
        }
        Commands::Test { urls, screenshots, retries, timeout } => {
            execute_multi_test(urls, screenshots, retries, timeout, &mut cost_tracker).await
        }
        Commands::Report => {
            println!("{}", cost_tracker.generate_daily_report());
            return Ok(());
        }
        Commands::Ask { command } => {
            execute_natural_language_command(&command, &mut cost_tracker, &config).await
        }
        Commands::Workflow { file, inputs, dry_run } => {
            execute_workflow(&file, inputs, dry_run, &mut cost_tracker).await
        }
        Commands::Extract { url, format, selector, output } => {
            execute_extraction(&url, &format, selector, output, &mut cost_tracker).await
        }
        Commands::Serve { port, docs } => {
            // Update config with the specified port
            let mut server_config = config.clone();
            if let Some(ref mut api) = server_config.api {
                api.port = Some(port);
            } else {
                server_config.api = Some(rainbow_poc::config::ApiConfig {
                    enabled: true,
                    port: Some(port),
                    auth_token: None,
                });
            }
            
            if docs {
                println!("📚 API documentation will be available at http://localhost:{}/docs", port);
            }
            
            println!("🚀 Starting API server on port {}...", port);
            println!("   Dashboard: http://localhost:{}/", port);
            println!("   Health check: http://localhost:{}/health", port);
            println!("   Metrics: http://localhost:{}/metrics", port);
            println!("   API endpoints: http://localhost:{}/", port);
            println!("   Press Ctrl+C to stop the server");
            
            start_server(server_config).await
        }
    };

    match result {
        Ok(_) => {
            info!("✅ Operation completed successfully!");
            println!("✅ Operation completed successfully!");
        }
        Err(e) => {
            error!("❌ Operation failed: {}", e);
            println!("❌ Operation failed: {}", e);
            println!("\n{}", cost_tracker.generate_daily_report());
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn execute_navigation(
    url: &str,
    screenshot: bool,
    filename: Option<String>,
    viewport_only: bool,
    width: u32,
    height: u32,
    cost_tracker: &mut CostTracker,
) -> Result<()> {
    let start_time = Utc::now();
    
    // Start browser operation
    info!("🚀 Starting browser...");
    let browser = SimpleBrowser::new().await?;

    let mut success = true;
    let mut error_message = String::new();

    // Execute navigation
    match browser.navigate_to(url).await {
        Ok(_) => {
            info!("✅ Navigation successful");
            
            // Get page title for confirmation
            match browser.get_title().await {
                Ok(title) => {
                    println!("📄 Page title: {}", title);
                    info!("Page title retrieved: {}", title);
                }
                Err(e) => {
                    warn!("Could not retrieve page title: {}", e);
                }
            }

            // Take screenshot if requested
            if screenshot {
                let filename = filename.unwrap_or_else(|| {
                    let safe_url = url.replace(['/', ':', '?'], "_");
                    format!("{}_{}.png", safe_url, Utc::now().format("%Y%m%d_%H%M%S"))
                });

                let screenshot_options = ScreenshotOptions {
                    full_page: !viewport_only,
                    viewport_width: width,
                    viewport_height: height,
                    wait_after_load: Duration::from_secs(2),
                };

                match browser.take_screenshot_with_options(&filename, &screenshot_options).await {
                    Ok(_) => {
                        println!("📸 Screenshot saved: screenshots/{}", filename);
                        info!("Screenshot saved successfully");
                    }
                    Err(e) => {
                        warn!("Screenshot failed: {}", e);
                        error_message = format!("Screenshot failed: {}", e);
                        // Don't mark as failure for screenshot issues
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Navigation failed: {}", e);
            error_message = format!("Navigation failed: {}", e);
            success = false;
        }
    }

    // Clean up browser
    if let Err(e) = browser.close().await {
        warn!("Error closing browser: {}", e);
    }

    // Record operation cost
    let actual_cost = cost_tracker.estimate_browser_operation_cost();
    let description = if screenshot {
        format!("Navigate to {} with screenshot", url)
    } else {
        format!("Navigate to {}", url)
    };

    cost_tracker.record_operation(
        "navigation".to_string(),
        description,
        actual_cost,
        success,
    )?;

    let duration = Utc::now().signed_duration_since(start_time);
    info!("Operation completed in {} seconds", duration.num_seconds());

    if success {
        Ok(())
    } else {
        Err(anyhow::anyhow!(error_message))
    }
}

async fn execute_multi_test(
    urls: Vec<String>,
    screenshots: bool,
    retries: u32,
    timeout_secs: u64,
    cost_tracker: &mut CostTracker,
) -> Result<()> {
    info!("🧪 Starting multi-website test with {} URLs", urls.len());
    
    if urls.is_empty() {
        return Err(anyhow::anyhow!("No URLs provided for testing"));
    }

    // Check if we can afford all operations
    let estimated_total_cost = cost_tracker.estimate_browser_operation_cost() * urls.len() as f64;
    if !cost_tracker.can_afford(estimated_total_cost) {
        error!("❌ Cannot afford all {} operations (${:.4})", urls.len(), estimated_total_cost);
        return Err(anyhow::anyhow!("Insufficient budget for multi-test"));
    }

    let mut results = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    // Create browser with custom timeout
    let browser = SimpleBrowser::new_with_config(retries, Duration::from_secs(timeout_secs)).await?;

    println!("🌐 Testing {} websites...\n", urls.len());

    for (i, url) in urls.iter().enumerate() {
        println!("📍 [{}/{}] Testing: {}", i + 1, urls.len(), url);
        
        let start_time = Utc::now();
        let mut test_success = true;
        let mut error_details = String::new();

        // Navigate with retries
        match browser.navigate_to_with_retry(url, retries).await {
            Ok(_) => {
                info!("✅ Navigation successful for {}", url);
                
                // Get page title
                match browser.get_title().await {
                    Ok(title) => {
                        println!("   📄 Title: {}", title);
                    }
                    Err(e) => {
                        warn!("Could not get title for {}: {}", url, e);
                    }
                }

                // Take screenshot if requested
                if screenshots {
                    let safe_url = url.replace(['/', ':', '?'], "_");
                    let filename = format!("test_{}_{}.png", safe_url, Utc::now().format("%H%M%S"));
                    
                    match browser.take_screenshot(&filename).await {
                        Ok(_) => {
                            println!("   📸 Screenshot: screenshots/{}", filename);
                        }
                        Err(e) => {
                            warn!("Screenshot failed for {}: {}", url, e);
                        }
                    }
                }

                successful += 1;
                println!("   ✅ Success");
            }
            Err(e) => {
                error!("❌ Navigation failed for {}: {}", url, e);
                error_details = format!("{}", e);
                test_success = false;
                failed += 1;
                println!("   ❌ Failed: {}", e);
            }
        }

        let duration = Utc::now().signed_duration_since(start_time);
        
        // Record operation
        let actual_cost = cost_tracker.estimate_browser_operation_cost();
        let description = if screenshots {
            format!("Multi-test {} with screenshot", url)
        } else {
            format!("Multi-test {}", url)
        };

        cost_tracker.record_operation(
            "multi_test".to_string(),
            description,
            actual_cost,
            test_success,
        )?;

        results.push((url.clone(), test_success, duration.num_seconds(), error_details));
        
        println!("   ⏱️  Duration: {}s\n", duration.num_seconds());

        // Small delay between requests to be respectful
        if i < urls.len() - 1 {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    }

    // Clean up browser
    if let Err(e) = browser.close().await {
        warn!("Error closing browser: {}", e);
    }

    // Print summary
    println!("📊 Test Summary:");
    println!("   ✅ Successful: {}", successful);
    println!("   ❌ Failed: {}", failed);
    println!("   📈 Success rate: {:.1}%", (successful as f64 / urls.len() as f64) * 100.0);
    println!("   💰 Total cost: ${:.4}", cost_tracker.estimate_browser_operation_cost() * urls.len() as f64);

    if failed > 0 {
        println!("\n❌ Failed URLs:");
        for (url, success, _duration, error) in &results {
            if !success {
                println!("   - {}: {}", url, error);
            }
        }
    }

    if failed > successful {
        Err(anyhow::anyhow!("More failures than successes in multi-test"))
    } else {
        Ok(())
    }
}

async fn execute_natural_language_command(
    user_command: &str,
    cost_tracker: &mut CostTracker,
    config: &Config,
) -> Result<()> {
    info!("🤖 Processing natural language command: {}", user_command);
    println!("🤖 Understanding your command...");

    // Load or create conversation context
    let mut context = ConversationContext::load_from_file()?;
    
    // Check if we have an OpenAI API key
    let api_key = match &config.openai_api_key() {
        Some(key) => key.clone(),
        None => {
            error!("OpenAI API key not found. Please set OPENAI_API_KEY environment variable.");
            println!("❌ OpenAI API key required for natural language commands.");
            println!("💡 Set OPENAI_API_KEY environment variable or use structured commands:");
            println!("   cargo run -- navigate google.com --screenshot");
            println!("   cargo run -- test --urls \"google.com,github.com\"");
            return Err(anyhow::anyhow!("OpenAI API key not configured"));
        }
    };

    // Initialize LLM service
    let llm_service = LLMService::new(api_key);
    
    // Check for similar commands in history
    let similar_commands = context.get_similar_commands(user_command);
    if !similar_commands.is_empty() {
        println!("💭 I found similar commands in your history:");
        for cmd in &similar_commands {
            println!("   - \"{}\": {}", cmd.user_input, cmd.execution_result.output_summary);
        }
    }

    let start_time = std::time::Instant::now();

    // Parse the natural language command with context
    let mut parsed_command = match llm_service.parse_natural_command(user_command, cost_tracker).await {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to parse command: {}", e);
            println!("❌ Sorry, I couldn't understand that command.");
            println!("💡 Try commands like:");
            println!("   \"navigate to google and take a screenshot\"");
            println!("   \"test google, github, and stackoverflow\"");
            println!("   \"show me the cost report\"");
            return Err(e);
        }
    };

    // Enhance command with user preferences and context
    context.enhance_command_with_context(&mut parsed_command);

    // Show what we understood
    let explanation = llm_service.explain_command(&parsed_command).await;
    println!("✅ I understood: {}", explanation);
    
    if parsed_command.confidence < 0.7 {
        println!("⚠️  I'm not very confident about this interpretation ({:.1}% sure)", 
                parsed_command.confidence * 100.0);
        println!("💡 You can continue or use structured commands instead.");
    }

    // Execute the parsed command
    let execution_result = match parsed_command.action.as_str() {
        "navigate" => {
            if let Some(url) = &parsed_command.url {
                match execute_navigation(
                    url,
                    parsed_command.screenshot,
                    parsed_command.filename.clone(),
                    parsed_command.viewport_only,
                    parsed_command.viewport_width.unwrap_or(1920),
                    parsed_command.viewport_height.unwrap_or(1080),
                    cost_tracker,
                ).await {
                    Ok(_) => {
                        let summary = if parsed_command.screenshot {
                            format!("Successfully navigated to {} and took screenshot", url)
                        } else {
                            format!("Successfully navigated to {}", url)
                        };
                        ExecutionResult {
                            success: true,
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            error_message: None,
                            output_summary: summary,
                        }
                    }
                    Err(e) => ExecutionResult {
                        success: false,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        error_message: Some(e.to_string()),
                        output_summary: format!("Failed to navigate to {}", url),
                    }
                }
            } else {
                ExecutionResult {
                    success: false,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some("No URL specified for navigation".to_string()),
                    output_summary: "Navigation command without URL".to_string(),
                }
            }
        }
        "test" => {
            if !parsed_command.urls.is_empty() {
                match execute_multi_test(
                    parsed_command.urls.clone(),
                    parsed_command.screenshot,
                    parsed_command.retries.unwrap_or(3),
                    parsed_command.timeout.unwrap_or(30),
                    cost_tracker,
                ).await {
                    Ok(_) => ExecutionResult {
                        success: true,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        error_message: None,
                        output_summary: format!("Successfully tested {} websites", parsed_command.urls.len()),
                    },
                    Err(e) => ExecutionResult {
                        success: false,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        error_message: Some(e.to_string()),
                        output_summary: format!("Multi-test failed for {} websites", parsed_command.urls.len()),
                    }
                }
            } else {
                ExecutionResult {
                    success: false,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some("No URLs specified for testing".to_string()),
                    output_summary: "Test command without URLs".to_string(),
                }
            }
        }
        "planning" => {
            // Use enhanced LLM service to get the task plan
            let task_understanding = MockTaskUnderstanding;
            match task_understanding.create_task_plan(user_command) {
                Ok(task_plan) => {
                    info!("Created task plan with {} steps", task_plan.steps.len());
                    
                    // Create TaskExecutor and execute the plan
                    let mut task_executor = TaskExecutor::new(cost_tracker.clone());
                    
                    match task_executor.execute_task_plan(&task_plan).await {
                        Ok(result) => {
                            // Update cost tracker from executor
                            // Note: In a production system, we'd need better state management here
                            
                            // Display results to user
                            task_executor.display_results(&result);
                            
                            ExecutionResult {
                                success: result.success,
                                duration_ms: result.total_duration_ms,
                                error_message: if result.success { None } else { Some("Task execution had failures".to_string()) },
                                output_summary: result.aggregated_results.summary,
                            }
                        }
                        Err(e) => {
                            error!("Task execution failed: {}", e);
                            println!("❌ Task execution failed: {}", e);
                            ExecutionResult {
                                success: false,
                                duration_ms: start_time.elapsed().as_millis() as u64,
                                error_message: Some(e.to_string()),
                                output_summary: "Task execution failed".to_string(),
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create task plan: {}", e);
                    println!("❌ Failed to create task plan: {}", e);
                    ExecutionResult {
                        success: false,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        error_message: Some(e.to_string()),
                        output_summary: "Task planning failed".to_string(),
                    }
                }
            }
        }
        "report" => {
            println!("\n{}", cost_tracker.generate_daily_report());
            ExecutionResult {
                success: true,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: None,
                output_summary: "Displayed cost report".to_string(),
            }
        }
        _ => {
            println!("❌ I'm not sure how to execute that command.");
            println!("💡 Supported commands:");
            println!("   - Navigate to websites");
            println!("   - Test multiple websites");
            println!("   - Show cost reports");
            ExecutionResult {
                success: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some("Unknown or unsupported command action".to_string()),
                output_summary: "Unsupported command".to_string(),
            }
        }
    };

    // Calculate cost for this operation (including LLM usage)
    let llm_cost = cost_tracker.estimate_llm_operation_cost(user_command.len());
    
    // Record this command in conversation history
    let history_entry = HistoryEntry {
        timestamp: Utc::now(),
        user_input: user_command.to_string(),
        parsed_command: parsed_command.clone(),
        execution_result: execution_result.clone(),
        cost: llm_cost,
    };

    context.add_history_entry(history_entry)?;

    // Show updated preferences if they changed
    let stats = context.get_stats();
    if stats.total_commands > 1 {
        println!("📊 Your preferences have been updated based on usage patterns");
        if context.preferences.default_screenshot {
            println!("   📸 Screenshots now default to ON");
        }
        if !context.preferences.favorite_sites.is_empty() {
            println!("   ⭐ Favorite sites: {}", context.preferences.favorite_sites.join(", "));
        }
    }

    if execution_result.success {
        Ok(())
    } else {
        Err(anyhow::anyhow!(execution_result.error_message.unwrap_or_else(|| "Command execution failed".to_string())))
    }
}

async fn execute_workflow(
    file_path: &str,
    inputs: Vec<String>,
    dry_run: bool,
    cost_tracker: &mut CostTracker,
) -> Result<()> {
    info!("🎭 Loading workflow from: {}", file_path);
    println!("🎭 Loading workflow from: {}", file_path);

    // Read workflow file
    let workflow_content = std::fs::read_to_string(file_path)
        .context(format!("Failed to read workflow file: {}", file_path))?;

    // Parse workflow based on file extension
    let workflow = if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        Workflow::from_yaml(&workflow_content)?
    } else if file_path.ends_with(".json") {
        Workflow::from_json(&workflow_content)?
    } else {
        // Try YAML first, then JSON
        Workflow::from_yaml(&workflow_content)
            .or_else(|_| Workflow::from_json(&workflow_content))
            .context("Failed to parse workflow as YAML or JSON")?
    };

    println!("📋 Workflow: {}", workflow.name);
    if let Some(desc) = &workflow.description {
        println!("📝 Description: {}", desc);
    }
    println!("📍 Steps: {}", workflow.steps.len());

    // Parse input variables
    let mut input_vars = HashMap::new();
    for input in inputs {
        if let Some((key, value)) = input.split_once('=') {
            input_vars.insert(key.to_string(), serde_json::json!(value));
        } else {
            warn!("Invalid input format: {} (expected key=value)", input);
        }
    }

    // Validate required inputs
    if let Some(input_defs) = &workflow.inputs {
        for input_def in input_defs {
            if input_def.required.unwrap_or(false) && !input_vars.contains_key(&input_def.name) {
                if let Some(default) = &input_def.default {
                    println!("ℹ️  Using default for '{}': {}", input_def.name, default);
                    input_vars.insert(input_def.name.clone(), default.clone());
                } else {
                    return Err(anyhow::anyhow!(
                        "Required input '{}' not provided. Use --inputs {}=value",
                        input_def.name, input_def.name
                    ));
                }
            }
        }
    }

    if dry_run {
        println!("🔍 Dry run mode - validating workflow without execution");
        println!("\n📊 Workflow Summary:");
        println!("   Name: {}", workflow.name);
        println!("   Steps: {}", workflow.steps.len());
        println!("   Parallel: {}", workflow.parallel.unwrap_or(false));
        println!("   Variables: {}", workflow.variables.len());
        println!("   Inputs provided: {}", input_vars.len());
        
        println!("\n📝 Steps:");
        for (i, step) in workflow.steps.iter().enumerate() {
            println!("   {}. {} - {:?}", i + 1, step.name, step.action);
        }
        
        println!("\n✅ Workflow validation successful!");
        return Ok(());
    }

    // Check budget
    let estimated_cost = cost_tracker.estimate_browser_operation_cost() * workflow.steps.len() as f64;
    if !cost_tracker.can_afford(estimated_cost) {
        error!("❌ Cannot afford workflow execution: ${:.4}", estimated_cost);
        println!("❌ Insufficient budget for workflow (estimated: ${:.4})", estimated_cost);
        return Err(anyhow::anyhow!("Insufficient budget"));
    }

    // Execute workflow
    println!("\n🚀 Executing workflow...\n");
    let start_time = std::time::Instant::now();
    
    let mut engine = WorkflowEngine::new(cost_tracker.clone());
    
    let result = match engine.execute_workflow(&workflow, Some(input_vars)).await {
        Ok(result) => result,
        Err(e) => {
            error!("Workflow execution failed: {}", e);
            println!("❌ Workflow execution failed: {}", e);
            
            // Update cost tracker with partial execution
            *cost_tracker = engine.cost_tracker;
            
            return Err(e);
        }
    };

    // Clean up browser first
    if let Err(e) = engine.cleanup().await {
        warn!("Failed to cleanup workflow engine: {}", e);
    }
    
    // Update cost tracker after cleanup
    *cost_tracker = engine.cost_tracker;

    let duration = start_time.elapsed();

    // Display results
    println!("\n📊 Workflow Results:");
    println!("   ✅ Success: {}", result.success);
    println!("   ⏱️  Duration: {:.2}s", duration.as_secs_f32());
    println!("   📍 Steps executed: {}/{}", result.steps_executed, workflow.steps.len());
    if result.steps_failed > 0 {
        println!("   ❌ Steps failed: {}", result.steps_failed);
    }
    println!("   💰 Cost: ${:.4}", result.cost);

    if !result.variables.is_empty() {
        println!("\n📦 Output Variables:");
        for (key, value) in &result.variables {
            if !key.starts_with('_') {  // Skip internal variables
                println!("   {}: {}", key, value);
            }
        }
    }

    if result.steps_failed > 0 {
        println!("\n❌ Failed Steps:");
        for entry in &result.execution_log {
            if !entry.success {
                println!("   - {}: {}", entry.step_name, entry.error.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
        }
    }

    if result.success {
        println!("\n✅ Workflow completed successfully!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Workflow failed with {} errors", result.steps_failed))
    }
}

async fn execute_extraction(
    url: &str,
    format: &str,
    selector: Option<String>,
    output: Option<String>,
    cost_tracker: &mut CostTracker,
) -> Result<()> {
    // Check budget
    let estimated_cost = cost_tracker.estimate_browser_operation_cost();
    if !cost_tracker.can_afford(estimated_cost) {
        error!("❌ Daily budget exceeded! Cannot proceed.");
        println!("\n{}", cost_tracker.generate_daily_report());
        return Ok(());
    }

    info!("📊 Starting data extraction from: {}", url);
    println!("📊 Extracting data from: {}", url);
    
    // Start browser
    let browser = SimpleBrowser::new_with_config(3, Duration::from_secs(30))
        .await
        .context("Failed to initialize browser")?;
    
    // Navigate to URL
    browser.navigate_to(url)
        .await
        .context("Failed to navigate to URL")?;
    
    // Wait for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create extractor
    let extractor = DataExtractor::new(browser.driver());
    
    // Extract data based on selector or all data
    let extracted_content = if let Some(sel) = selector {
        info!("Extracting using selector: {}", sel);
        println!("🔍 Using selector: {}", sel);
        
        match format.to_lowercase().as_str() {
            "json" => {
                let results = extractor.extract_by_selector(&sel)
                    .await
                    .context("Failed to extract with selector")?;
                serde_json::to_string_pretty(&results)?
            }
            "csv" | "text" => {
                let results = extractor.extract_by_selector(&sel)
                    .await
                    .context("Failed to extract with selector")?;
                results.join("\n")
            }
            _ => {
                println!("⚠️  Unknown format '{}', using JSON", format);
                let results = extractor.extract_by_selector(&sel)
                    .await
                    .context("Failed to extract with selector")?;
                serde_json::to_string_pretty(&results)?
            }
        }
    } else {
        // Extract all data
        info!("Extracting all data from page");
        println!("📋 Extracting all data...");
        
        match format.to_lowercase().as_str() {
            "json" => {
                extractor.extract_as_json()
                    .await
                    .context("Failed to extract as JSON")?
            }
            "csv" => {
                extractor.extract_links_as_csv()
                    .await
                    .context("Failed to extract as CSV")?
            }
            "text" => {
                let data = extractor.extract_all()
                    .await
                    .context("Failed to extract data")?;
                
                // Format as readable text
                let mut text = String::new();
                text.push_str(&format!("Title: {}\n", data.title.unwrap_or_default()));
                text.push_str(&format!("URL: {}\n", data.url));
                
                if let Some(desc) = data.meta_description {
                    text.push_str(&format!("Description: {}\n", desc));
                }
                
                text.push_str("\n=== Headings ===\n");
                for heading in &data.headings {
                    text.push_str(&format!("- {}\n", heading));
                }
                
                text.push_str("\n=== Content ===\n");
                for para in &data.text_content {
                    text.push_str(&format!("{}\n\n", para));
                }
                
                text.push_str(&format!("\n=== Links ({}) ===\n", data.links.len()));
                for link in &data.links {
                    text.push_str(&format!("- {} -> {}\n", link.text, link.href));
                }
                
                text
            }
            _ => {
                println!("⚠️  Unknown format '{}', using JSON", format);
                extractor.extract_as_json()
                    .await
                    .context("Failed to extract as JSON")?
            }
        }
    };
    
    // Output results
    if let Some(output_file) = output {
        std::fs::write(&output_file, &extracted_content)
            .context("Failed to write output file")?;
        println!("✅ Data extracted and saved to: {}", output_file);
        info!("Extraction saved to: {}", output_file);
    } else {
        // Print to stdout
        println!("\n{}", extracted_content);
    }
    
    // Record operation
    cost_tracker.record_operation(
        "extract".to_string(),
        format!("Extract data from {}", url),
        estimated_cost,
        true,
    )?;
    
    // Close browser
    browser.close().await?;
    
    println!("\n✅ Extraction completed successfully!");
    Ok(())
}
