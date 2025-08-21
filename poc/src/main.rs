use clap::{Parser, Subcommand};
use rainbow_poc::{SimpleBrowser, CostTracker, Config, ScreenshotOptions, LLMService, ConversationContext, HistoryEntry, ExecutionResult, Workflow, WorkflowEngine, start_server, DataExtractor, TaskExecutor};
use anyhow::{Result, Context};
use tracing::{info, error, warn};
use chrono::Utc;
use std::time::Duration;
use std::collections::HashMap;

// Import the tools module for our browser automation tools
use rainbow_poc::tools::{Tool, ToolInput, ToolOutput};
use rainbow_poc::tools::navigation::{NavigateToUrl, NavigateInput, ScrollPage, ScrollInput, ScrollDirection};
use rainbow_poc::tools::interaction::{Click, ClickInput, TypeText, TypeTextInput, SelectOption, SelectOptionInput};
use rainbow_poc::tools::synchronization::{WaitForElement, WaitForElementInput, ElementState, WaitStrategy, WaitForCondition, WaitForConditionInput, WaitCondition};
use rainbow_poc::tools::data_extraction::{ExtractText, ExtractTextInput, TextExtractionType, OutputFormat, ExtractionScope, ExtractionConfig, ExtractData, ExtractDataInput, DataType, SelectorType, DataField, ExtractionTemplate, ExtractTable, ExtractTableInput, TableDataType, TableExtractionConfig, ExtractForm, ExtractFormInput, FormExtractionConfig, ExtractLinks, ExtractLinksInput, LinkExtractionConfig};
use rainbow_poc::tools::advanced_automation::{SmartActions, SmartActionsInput, SmartActionConfig, WorkflowOrchestrator, WorkflowOrchestratorInput, WorkflowDefinition, WorkflowConfig, VisualValidator, VisualValidatorInput, VisualValidationConfig, VisualTestType, ViewportSize, PerformanceMonitor, PerformanceMonitorInput, PerformanceConfig};

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
    
    /// Click on an element on a webpage
    Click {
        /// URL to navigate to
        url: String,
        
        /// CSS selector of the element to click
        #[arg(short, long)]
        selector: String,
        
        /// Wait time in seconds before clicking
        #[arg(short, long, default_value = "2")]
        wait: u64,
    },
    
    /// Type text into an input field
    Type {
        /// URL to navigate to
        url: String,
        
        /// CSS selector of the input field
        #[arg(short, long)]
        selector: String,
        
        /// Text to type
        #[arg(short, long)]
        text: String,
        
        /// Clear the field before typing
        #[arg(short, long, default_value = "true")]
        clear: bool,
    },
    
    /// Select an option from a dropdown
    Select {
        /// URL to navigate to
        url: String,
        
        /// CSS selector of the select element
        #[arg(short, long)]
        selector: String,
        
        /// Value or text of the option to select
        #[arg(short, long)]
        value: String,
    },
    
    /// Scroll on a webpage
    Scroll {
        /// URL to navigate to
        url: String,
        
        /// Scroll direction or amount (up/down/top/bottom or pixels like "500")
        #[arg(short, long, default_value = "down")]
        direction: String,
        
        /// Use smooth scrolling
        #[arg(long)]
        smooth: bool,
    },
    
    /// Wait for an element to reach a specific state
    WaitForElement {
        /// URL to navigate to
        url: String,
        
        /// CSS selector of the element to wait for
        #[arg(short, long)]
        selector: String,
        
        /// Element state to wait for (attached, detached, visible, hidden, enabled, disabled)
        #[arg(long, default_value = "visible")]
        state: String,
        
        /// Expected text content (optional)
        #[arg(long)]
        text: Option<String>,
        
        /// Attribute name to check (optional)
        #[arg(long)]
        attribute: Option<String>,
        
        /// Expected attribute value (requires --attribute)
        #[arg(long)]
        attribute_value: Option<String>,
        
        /// Timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
        
        /// Polling interval in milliseconds
        #[arg(long, default_value = "100")]
        interval: u64,
    },
    
    /// Wait for a custom condition to be met
    WaitForCondition {
        /// URL to navigate to
        url: String,
        
        /// Condition type (url-contains, url-equals, title-contains, title-equals, element-count, custom-js)
        #[arg(short, long)]
        condition: String,
        
        /// Expected value for the condition
        #[arg(short, long)]
        value: String,
        
        /// CSS selector (required for element-count)
        #[arg(short, long)]
        selector: Option<String>,
        
        /// Expected count (required for element-count)
        #[arg(long)]
        count: Option<usize>,
        
        /// Timeout in milliseconds
        #[arg(long, default_value = "30000")]
        timeout: u64,
        
        /// Polling interval in milliseconds
        #[arg(long, default_value = "100")]
        interval: u64,
    },
    
    /// Extract text content from a webpage with multiple output formats
    ExtractText {
        /// URL to extract text from
        url: String,
        
        /// CSS selector for elements to extract text from (optional, defaults to body)
        #[arg(short, long)]
        selector: Option<String>,
        
        /// Type of text extraction (innertext, textcontent, innerhtml, outerhtml, all, attr:<name>)
        #[arg(short, long, default_value = "innertext")]
        extraction_type: String,
        
        /// Output format (json, text, csv, html, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Extract from multiple elements instead of just the first match
        #[arg(short, long)]
        multiple: bool,
        
        /// Include metadata in the output
        #[arg(long)]
        include_metadata: bool,
        
        /// Clean/normalize extracted text
        #[arg(long, default_value = "true")]
        clean: bool,
        
        /// Maximum number of items to extract (0 = no limit)
        #[arg(long, default_value = "0")]
        max_items: usize,
        
        /// Minimum text length to include
        #[arg(long)]
        min_length: Option<usize>,
        
        /// Maximum text length per item
        #[arg(long)]
        max_length: Option<usize>,
        
        /// Text filters (case-insensitive contains, comma-separated)
        #[arg(long, value_delimiter = ',')]
        filters: Option<Vec<String>>,
        
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Extract structured data using templates and advanced selectors
    ExtractData {
        /// URL to extract data from
        url: String,
        
        /// Template file (JSON/YAML) or inline template definition
        #[arg(short, long)]
        template: Option<String>,
        
        /// Field definitions as key:selector:type (can be repeated)
        #[arg(long, value_delimiter = ',')]
        fields: Option<Vec<String>>,
        
        /// Root selector for multiple record extraction
        #[arg(short, long)]
        root_selector: Option<String>,
        
        /// Extract multiple records instead of single record
        #[arg(short, long)]
        multiple: bool,
        
        /// Maximum number of records to extract
        #[arg(long)]
        max_records: Option<usize>,
        
        /// Output format (json, text, csv, html, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Include metadata in the output
        #[arg(long)]
        include_metadata: bool,
        
        /// Validate extracted data against schema
        #[arg(long)]
        validate: bool,
        
        /// Skip invalid records instead of failing
        #[arg(long)]
        skip_invalid: bool,
        
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Extract tabular data with intelligent table structure detection
    ExtractTable {
        /// URL to extract table data from
        url: String,
        
        /// CSS selector for table elements (optional, defaults to "table")
        #[arg(short, long)]
        table_selector: Option<String>,
        
        /// Extract from multiple tables instead of just the first
        #[arg(short, long)]
        multiple: bool,
        
        /// Column data types as column_name:type (can be repeated)
        #[arg(long, value_delimiter = ',')]
        column_types: Option<Vec<String>>,
        
        /// Auto-infer column data types from content
        #[arg(long, default_value = "true")]
        auto_infer: bool,
        
        /// Include table headers in output
        #[arg(long, default_value = "true")]
        include_headers: bool,
        
        /// Include table footers in output
        #[arg(long)]
        include_footers: bool,
        
        /// Skip completely empty rows
        #[arg(long, default_value = "true")]
        skip_empty_rows: bool,
        
        /// Minimum number of columns required for valid table
        #[arg(long, default_value = "2")]
        min_columns: usize,
        
        /// Maximum number of rows to extract per table (0 = no limit)
        #[arg(long, default_value = "0")]
        max_rows: usize,
        
        /// Column name mapping as old_name:new_name (can be repeated)
        #[arg(long, value_delimiter = ',')]
        column_mapping: Option<Vec<String>>,
        
        /// Include only these columns (comma-separated)
        #[arg(long, value_delimiter = ',')]
        include_columns: Option<Vec<String>>,
        
        /// Exclude these columns (comma-separated)
        #[arg(long, value_delimiter = ',')]
        exclude_columns: Option<Vec<String>>,
        
        /// Output format (json, csv, text, html, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Include metadata in the output
        #[arg(long)]
        include_metadata: bool,
        
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Extract comprehensive form information with validation analysis
    ExtractForm {
        /// URL to extract forms from
        url: String,
        
        /// CSS selector for form elements (optional, defaults to "form")
        #[arg(short, long)]
        form_selector: Option<String>,
        
        /// Extract from multiple forms instead of just the first
        #[arg(short, long)]
        multiple: bool,
        
        /// Include hidden form fields
        #[arg(long)]
        include_hidden: bool,
        
        /// Include disabled form fields
        #[arg(long, default_value = "true")]
        include_disabled: bool,
        
        /// Extract field options (for select/radio/checkbox)
        #[arg(long, default_value = "true")]
        extract_options: bool,
        
        /// Analyze form validation rules
        #[arg(long, default_value = "true")]
        analyze_validation: bool,
        
        /// Include field positioning information
        #[arg(long)]
        include_positioning: bool,
        
        /// Maximum options per field to extract
        #[arg(long, default_value = "100")]
        max_options_per_field: usize,
        
        /// Output format (json, text, csv, html, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Include metadata in the output
        #[arg(long)]
        include_metadata: bool,
        
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Extract and analyze links with categorization and validation
    ExtractLinks {
        /// URL to extract links from
        url: String,
        
        /// CSS selector for link elements (optional, defaults to "a[href]")
        #[arg(short, long)]
        link_selector: Option<String>,
        
        /// Include anchor links (#fragment)
        #[arg(long, default_value = "true")]
        include_anchors: bool,
        
        /// Include email links (mailto:)
        #[arg(long, default_value = "true")]
        include_email: bool,
        
        /// Include phone links (tel:)
        #[arg(long, default_value = "true")]
        include_phone: bool,
        
        /// Include JavaScript links (javascript:)
        #[arg(long)]
        include_javascript: bool,
        
        /// Validate URLs and parse components
        #[arg(long, default_value = "true")]
        validate_urls: bool,
        
        /// Categorize links by type
        #[arg(long, default_value = "true")]
        categorize_links: bool,
        
        /// Include link positioning information
        #[arg(long)]
        include_positioning: bool,
        
        /// Analyze link accessibility
        #[arg(long, default_value = "true")]
        analyze_accessibility: bool,
        
        /// Minimum link text length to include
        #[arg(long, default_value = "0")]
        min_text_length: usize,
        
        /// Maximum number of links to extract (0 = no limit)
        #[arg(long, default_value = "0")]
        max_links: usize,
        
        /// Additional internal domains (comma-separated)
        #[arg(long, value_delimiter = ',')]
        internal_domains: Option<Vec<String>>,
        
        /// Text patterns to exclude (comma-separated)
        #[arg(long, value_delimiter = ',')]
        exclude_patterns: Option<Vec<String>>,
        
        /// Output format (json, text, csv, html, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Include metadata in the output
        #[arg(long)]
        include_metadata: bool,
        
        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Intelligent form filling and context-aware automation
    SmartActions {
        /// URL containing the form to fill
        url: String,
        
        /// CSS selector for the form (optional, defaults to first form)
        #[arg(short, long)]
        form_selector: Option<String>,
        
        /// Form data as key=value pairs (can be repeated)
        #[arg(long, value_delimiter = ',')]
        form_data: Option<Vec<String>>,
        
        /// User preferences for smart suggestions as key=value pairs
        #[arg(long, value_delimiter = ',')]
        user_preferences: Option<Vec<String>>,
        
        /// Custom field mappings as old_name:new_name (can be repeated)
        #[arg(long, value_delimiter = ',')]
        field_mappings: Option<Vec<String>>,
        
        /// Use intelligent field type detection
        #[arg(long, default_value = "true")]
        intelligent_detection: bool,
        
        /// Use smart value suggestions for empty fields
        #[arg(long, default_value = "true")]
        smart_suggestions: bool,
        
        /// Validate fields before filling
        #[arg(long, default_value = "true")]
        validate_before_fill: bool,
        
        /// Wait for element visibility before interaction
        #[arg(long, default_value = "true")]
        wait_for_visibility: bool,
        
        /// Maximum wait time in milliseconds
        #[arg(long, default_value = "10000")]
        max_wait_time: u64,
        
        /// Number of retry attempts for failed actions
        #[arg(long, default_value = "3")]
        retry_attempts: usize,
        
        /// Automatically submit form after filling
        #[arg(long)]
        auto_submit: bool,
        
        /// Output file for automation results (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Execute complex automation workflows with conditional logic and branching
    WorkflowOrchestrator {
        /// URL to start the workflow
        url: String,
        
        /// Workflow definition file (JSON format)
        #[arg(short, long)]
        workflow_file: String,
        
        /// Initial variables as key=value pairs
        #[arg(long, value_delimiter = ',')]
        variables: Option<Vec<String>>,
        
        /// Resume from saved workflow state
        #[arg(long)]
        resume_state: Option<String>,
        
        /// Capture screenshots at each step
        #[arg(long)]
        capture_screenshots: bool,
        
        /// Save workflow state at each step
        #[arg(long, default_value = "true")]
        save_state: bool,
        
        /// Enable parallel execution where possible
        #[arg(long)]
        enable_parallel: bool,
        
        /// Strict condition validation
        #[arg(long, default_value = "true")]
        strict_validation: bool,
        
        /// Maximum parallel executions
        #[arg(long, default_value = "3")]
        max_parallel: usize,
        
        /// Step execution delay in milliseconds
        #[arg(long, default_value = "500")]
        step_delay_ms: u64,
        
        /// Enable detailed execution logging
        #[arg(long, default_value = "true")]
        detailed_logging: bool,
        
        /// Output file for workflow results
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Perform comprehensive UI testing and visual validation
    VisualValidator {
        /// URL to perform visual validation on
        url: String,
        
        /// Visual test types to perform (screenshot-comparison, element-validation, responsive-validation, etc.)
        #[arg(long, value_delimiter = ',')]
        test_types: Option<Vec<String>>,
        
        /// Baseline screenshot path for comparison tests
        #[arg(long)]
        baseline_path: Option<String>,
        
        /// Output directory for screenshots and reports
        #[arg(long, default_value = "visual_validation_output")]
        output_directory: String,
        
        /// Element selectors to validate specifically (can be repeated)
        #[arg(long, value_delimiter = ',')]
        target_elements: Option<Vec<String>>,
        
        /// Viewport sizes to test for responsive validation (WIDTHxHEIGHT format)
        #[arg(long, value_delimiter = ',')]
        viewport_sizes: Option<Vec<String>>,
        
        /// Similarity threshold for pass/fail (0.0-100.0)
        #[arg(long, default_value = "98.0")]
        similarity_threshold: f64,
        
        /// Pixel difference tolerance
        #[arg(long, default_value = "100")]
        pixel_tolerance: u32,
        
        /// Color difference tolerance
        #[arg(long, default_value = "5.0")]
        color_tolerance: f64,
        
        /// Enable anti-aliasing compensation
        #[arg(long, default_value = "true")]
        anti_aliasing_tolerance: bool,
        
        /// Filter dynamic content from comparisons
        #[arg(long, default_value = "true")]
        filter_dynamic_content: bool,
        
        /// Generate detailed HTML reports
        #[arg(long, default_value = "true")]
        generate_reports: bool,
        
        /// Save difference images highlighting changes
        #[arg(long, default_value = "true")]
        save_differences: bool,
        
        /// Image quality for screenshots (1-100)
        #[arg(long, default_value = "90")]
        image_quality: u8,
        
        /// Output file for validation results
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Monitor and analyze comprehensive performance metrics
    PerformanceMonitor {
        /// URL to monitor performance
        #[arg(long)]
        url: Option<String>,
        
        /// Number of measurement iterations
        #[arg(long, default_value = "1")]
        iterations: usize,
        
        /// Wait time after page load (milliseconds)
        #[arg(long, default_value = "3000")]
        wait_after_load: u64,
        
        /// Interval between measurements (milliseconds)
        #[arg(long, default_value = "1000")]
        interval: u64,
        
        /// Capture Core Web Vitals
        #[arg(long, default_value = "true")]
        capture_core_web_vitals: bool,
        
        /// Capture resource timing
        #[arg(long, default_value = "true")]
        capture_resource_timing: bool,
        
        /// Capture memory metrics
        #[arg(long, default_value = "true")]
        capture_memory_metrics: bool,
        
        /// Capture network metrics
        #[arg(long, default_value = "true")]
        capture_network_metrics: bool,
        
        /// Generate detailed performance report
        #[arg(long, default_value = "true")]
        generate_report: bool,
        
        /// Output file for performance results
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
        Commands::Click { url, selector, wait } => {
            execute_click(&url, &selector, wait, &mut cost_tracker).await
        }
        Commands::Type { url, selector, text, clear } => {
            execute_type(&url, &selector, &text, clear, &mut cost_tracker).await
        }
        Commands::Select { url, selector, value } => {
            execute_select(&url, &selector, &value, &mut cost_tracker).await
        }
        Commands::Scroll { url, direction, smooth } => {
            execute_scroll(&url, &direction, smooth, &mut cost_tracker).await
        }
        Commands::WaitForElement { url, selector, state, text, attribute, attribute_value, timeout, interval } => {
            execute_wait_for_element(&url, &selector, &state, text.as_deref(), attribute.as_deref(), attribute_value.as_deref(), timeout, interval, &mut cost_tracker).await
        }
        Commands::WaitForCondition { url, condition, value, selector, count, timeout, interval } => {
            execute_wait_for_condition(&url, &condition, &value, selector.as_deref(), count, timeout, interval, &mut cost_tracker).await
        }
        Commands::ExtractText { 
            url, selector, extraction_type, format, multiple, include_metadata, clean, 
            max_items, min_length, max_length, filters, output 
        } => {
            execute_extract_text(&url, selector.as_deref(), &extraction_type, &format, multiple, include_metadata, clean, max_items, min_length, max_length, filters.as_ref(), output.as_deref(), &mut cost_tracker).await
        }
        Commands::ExtractData { 
            url, template, fields, root_selector, multiple, max_records, format, 
            include_metadata, validate, skip_invalid, output 
        } => {
            execute_extract_data(&url, template.as_deref(), fields.as_ref(), root_selector.as_deref(), multiple, max_records, &format, include_metadata, validate, skip_invalid, output.as_deref(), &mut cost_tracker).await
        }
        Commands::ExtractTable {
            url, table_selector, multiple, column_types, auto_infer, include_headers, 
            include_footers, skip_empty_rows, min_columns, max_rows, column_mapping, 
            include_columns, exclude_columns, format, include_metadata, output
        } => {
            execute_extract_table(&url, table_selector.as_deref(), multiple, column_types.as_ref(), auto_infer, include_headers, include_footers, skip_empty_rows, min_columns, max_rows, column_mapping.as_ref(), include_columns.as_ref(), exclude_columns.as_ref(), &format, include_metadata, output.as_deref(), &mut cost_tracker).await
        }
        Commands::ExtractForm {
            url, form_selector, multiple, include_hidden, include_disabled, extract_options,
            analyze_validation, include_positioning, max_options_per_field, format, 
            include_metadata, output
        } => {
            execute_extract_form(&url, form_selector.as_deref(), multiple, include_hidden, include_disabled, extract_options, analyze_validation, include_positioning, max_options_per_field, &format, include_metadata, output.as_deref(), &mut cost_tracker).await
        }
        Commands::ExtractLinks {
            url, link_selector, include_anchors, include_email, include_phone, include_javascript,
            validate_urls, categorize_links, include_positioning, analyze_accessibility,
            min_text_length, max_links, internal_domains, exclude_patterns, format,
            include_metadata, output
        } => {
            execute_extract_links(&url, link_selector.as_deref(), include_anchors, include_email, include_phone, include_javascript, validate_urls, categorize_links, include_positioning, analyze_accessibility, min_text_length, max_links, internal_domains.as_ref(), exclude_patterns.as_ref(), &format, include_metadata, output.as_deref(), &mut cost_tracker).await
        }
        Commands::SmartActions {
            url, form_selector, form_data, user_preferences, field_mappings,
            intelligent_detection, smart_suggestions, validate_before_fill, wait_for_visibility,
            max_wait_time, retry_attempts, auto_submit, output
        } => {
            execute_smart_actions(&url, form_selector.as_deref(), form_data.as_ref(), user_preferences.as_ref(), field_mappings.as_ref(), intelligent_detection, smart_suggestions, validate_before_fill, wait_for_visibility, max_wait_time, retry_attempts, auto_submit, output.as_deref(), &mut cost_tracker).await
        }
        Commands::WorkflowOrchestrator {
            url, workflow_file, variables, resume_state, capture_screenshots, save_state,
            enable_parallel, strict_validation, max_parallel, step_delay_ms, detailed_logging, output
        } => {
            execute_workflow_orchestrator(&url, &workflow_file, variables.as_ref(), resume_state.as_deref(), capture_screenshots, save_state, enable_parallel, strict_validation, max_parallel, step_delay_ms, detailed_logging, output.as_deref(), &mut cost_tracker).await
        }
        Commands::VisualValidator {
            url, test_types, baseline_path, output_directory, target_elements, viewport_sizes,
            similarity_threshold, pixel_tolerance, color_tolerance, anti_aliasing_tolerance,
            filter_dynamic_content, generate_reports, save_differences, image_quality, output
        } => {
            execute_visual_validator(&url, test_types.as_ref(), baseline_path.as_deref(), &output_directory, target_elements.as_ref(), viewport_sizes.as_ref(), similarity_threshold, pixel_tolerance, color_tolerance, anti_aliasing_tolerance, filter_dynamic_content, generate_reports, save_differences, image_quality, output.as_deref(), &mut cost_tracker).await
        }
        Commands::PerformanceMonitor {
            url, iterations, wait_after_load, interval, capture_core_web_vitals,
            capture_resource_timing, capture_memory_metrics, capture_network_metrics,
            generate_report, output
        } => {
            execute_performance_monitor(url.as_deref(), iterations, wait_after_load, interval, capture_core_web_vitals, capture_resource_timing, capture_memory_metrics, capture_network_metrics, generate_report, output.as_deref(), &mut cost_tracker).await
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
            // Use simplified organic intelligence to get the task plan  
            use rainbow_poc::{SimpleOrganicPerception, TaskUnderstanding};
            
            let task_understanding = SimpleOrganicPerception::from_env();
            info!("🧠 Using simplified organic intelligence for task planning");
            
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

// ========================================
// Tool Execution Functions for 5 Tools
// ========================================

/// Execute the Click tool
async fn execute_click(url: &str, selector: &str, wait: u64, cost_tracker: &mut CostTracker) -> Result<()> {
    println!("🖱️ Executing Click Tool");
    println!("   URL: {}", url);
    println!("   Selector: {}", selector);
    println!("   Wait: {} seconds", wait);
    
    // Create browser instance
    let browser = SimpleBrowser::new().await?;
    
    // Navigate to URL first
    browser.navigate_to(url).await?;
    println!("✅ Navigated to {}", url);
    
    // Wait before clicking
    if wait > 0 {
        println!("⏳ Waiting {} seconds...", wait);
        tokio::time::sleep(Duration::from_secs(wait)).await;
    }
    
    // Create and execute the Click tool
    let click_tool = Click::new(std::sync::Arc::new(browser.clone()));
    let input = ClickInput {
        selector: selector.to_string(),
        wait_for_navigation: Some(false),
        timeout: Some(Duration::from_secs(10)),
    };
    
    println!("🖱️ Clicking on element: {}", selector);
    click_tool.execute(input).await?;
    println!("✅ Click executed successfully!");
    
    // Take a screenshot to show the result
    let screenshot_name = format!("click_result_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    browser.take_screenshot(&screenshot_name).await?;
    println!("📸 Screenshot saved: screenshots/{}", screenshot_name);
    
    // Record operation
    cost_tracker.record_operation(
        "click".to_string(),
        format!("Click on {} at {}", selector, url),
        0.01,
        true,
    )?;
    
    // Close browser
    browser.close().await?;
    Ok(())
}

/// Execute the Type tool
async fn execute_type(url: &str, selector: &str, text: &str, clear: bool, cost_tracker: &mut CostTracker) -> Result<()> {
    println!("⌨️ Executing Type Tool");
    println!("   URL: {}", url);
    println!("   Selector: {}", selector);
    println!("   Text: {}", text);
    println!("   Clear first: {}", clear);
    
    // Create browser instance
    let browser = SimpleBrowser::new().await?;
    
    // Navigate to URL first
    browser.navigate_to(url).await?;
    println!("✅ Navigated to {}", url);
    
    // Wait a moment for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create and execute the TypeText tool
    let type_tool = TypeText::new(std::sync::Arc::new(browser.clone()));
    let input = TypeTextInput {
        selector: selector.to_string(),
        text: text.to_string(),
        clear_first: clear,
        typing_delay_ms: Some(50),
    };
    
    println!("⌨️ Typing text into: {}", selector);
    type_tool.execute(input).await?;
    println!("✅ Text typed successfully!");
    
    // Take a screenshot to show the result
    let screenshot_name = format!("type_result_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    browser.take_screenshot(&screenshot_name).await?;
    println!("📸 Screenshot saved: screenshots/{}", screenshot_name);
    
    // Record operation
    cost_tracker.record_operation(
        "type".to_string(),
        format!("Type text into {} at {}", selector, url),
        0.01,
        true,
    )?;
    
    // Close browser
    browser.close().await?;
    Ok(())
}

/// Execute the Select tool
async fn execute_select(url: &str, selector: &str, value: &str, cost_tracker: &mut CostTracker) -> Result<()> {
    println!("📋 Executing Select Tool");
    println!("   URL: {}", url);
    println!("   Selector: {}", selector);
    println!("   Value: {}", value);
    
    // Create browser instance
    let browser = SimpleBrowser::new().await?;
    
    // Navigate to URL first
    browser.navigate_to(url).await?;
    println!("✅ Navigated to {}", url);
    
    // Wait a moment for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create and execute the SelectOption tool
    let select_tool = SelectOption::new(std::sync::Arc::new(browser.clone()));
    let input = SelectOptionInput {
        selector: selector.to_string(),
        value: Some(value.to_string()),
        text: None,
        index: None,
    };
    
    println!("📋 Selecting option: {}", value);
    select_tool.execute(input).await?;
    println!("✅ Option selected successfully!");
    
    // Take a screenshot to show the result
    let screenshot_name = format!("select_result_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    browser.take_screenshot(&screenshot_name).await?;
    println!("📸 Screenshot saved: screenshots/{}", screenshot_name);
    
    // Record operation
    cost_tracker.record_operation(
        "select".to_string(),
        format!("Select {} in {} at {}", value, selector, url),
        0.01,
        true,
    )?;
    
    // Close browser
    browser.close().await?;
    Ok(())
}

/// Execute the Scroll tool
async fn execute_scroll(url: &str, direction: &str, smooth: bool, cost_tracker: &mut CostTracker) -> Result<()> {
    println!("📜 Executing Scroll Tool");
    println!("   URL: {}", url);
    println!("   Direction: {}", direction);
    println!("   Smooth: {}", smooth);
    
    // Create browser instance
    let browser = SimpleBrowser::new().await?;
    
    // Navigate to URL first
    browser.navigate_to(url).await?;
    println!("✅ Navigated to {}", url);
    
    // Wait a moment for page to load
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Create and execute the ScrollPage tool
    let scroll_tool = ScrollPage::new(std::sync::Arc::new(browser.clone()));
    
    // Parse direction
    let scroll_dir = match direction.to_lowercase().as_str() {
        "up" => ScrollDirection::Up,
        "down" => ScrollDirection::Down,
        "top" => ScrollDirection::Top,
        "bottom" => ScrollDirection::Bottom,
        pixels if pixels.parse::<i32>().is_ok() => {
            ScrollDirection::ToPosition { 
                x: 0, 
                y: pixels.parse().unwrap() 
            }
        }
        _ => ScrollDirection::Down, // Default to down
    };
    
    let input = ScrollInput {
        direction: scroll_dir,
        smooth: Some(smooth),
        duration_ms: if smooth { Some(1000) } else { None },
    };
    
    println!("📜 Scrolling page...");
    scroll_tool.execute(input).await?;
    println!("✅ Scroll executed successfully!");
    
    // Take a screenshot to show the result
    let screenshot_name = format!("scroll_result_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    browser.take_screenshot(&screenshot_name).await?;
    println!("📸 Screenshot saved: screenshots/{}", screenshot_name);
    
    // Record operation
    cost_tracker.record_operation(
        "scroll".to_string(),
        format!("Scroll {} at {}", direction, url),
        0.01,
        true,
    )?;
    
    // Close browser
    browser.close().await?;
    Ok(())
}

async fn execute_wait_for_element(
    url: &str,
    selector: &str,
    state: &str,
    text: Option<&str>,
    attribute: Option<&str>,
    attribute_value: Option<&str>,
    timeout: u64,
    interval: u64,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("🔍 Waiting for element '{}' to be {} on {}", selector, state, url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Convert state string to ElementState enum
    let element_state = match state.to_lowercase().as_str() {
        "attached" => ElementState::Attached,
        "detached" => ElementState::Detached,
        "visible" => ElementState::Visible,
        "hidden" => ElementState::Hidden,
        "enabled" => ElementState::Enabled,
        "disabled" => ElementState::Disabled,
        _ => {
            return Err(anyhow::anyhow!("Invalid state '{}'. Valid states: attached, detached, visible, hidden, enabled, disabled", state));
        }
    };
    
    // Create wait strategy
    let wait_strategy = WaitStrategy {
        timeout_ms: timeout,
        poll_interval_ms: interval,
        ..Default::default()
    };
    
    // Create wait tool
    let wait_tool = WaitForElement::new(Arc::new(browser.driver().clone()));
    
    // Create input
    let input = WaitForElementInput {
        selector: selector.to_string(),
        state: element_state.clone(),
        strategy: Some(wait_strategy),
        text_content: text.map(|s| s.to_string()),
        attribute_name: attribute.map(|s| s.to_string()),
        attribute_value: attribute_value.map(|s| s.to_string()),
    };
    
    println!("⏳ Starting wait with timeout: {}ms, interval: {}ms", timeout, interval);
    
    // Execute the wait operation
    let result = wait_tool.execute(input).await?;
    
    if result.success {
        println!("✅ Element condition met successfully!");
        println!("   - Wait time: {}ms", result.wait_time_ms);
        println!("   - Attempts: {}", result.attempts);
        if let Some(final_state) = result.final_state {
            println!("   - Final state: {:?}", final_state);
        }
    } else {
        println!("❌ Wait condition not met:");
        println!("   - Wait time: {}ms", result.wait_time_ms);
        println!("   - Attempts: {}", result.attempts);
        if let Some(error) = result.error_message {
            println!("   - Error: {}", error);
        }
    }
    
    // Take a screenshot to show the final state
    let screenshot_name = format!("wait_element_result_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    browser.take_screenshot(&screenshot_name).await?;
    println!("📸 Screenshot saved: screenshots/{}", screenshot_name);
    
    // Record operation
    cost_tracker.record_operation(
        "wait_for_element".to_string(),
        format!("Wait for '{}' to be {} at {}", selector, state, url),
        0.005, // Slightly higher cost due to polling
        result.success,
    )?;
    
    // Close browser
    browser.close().await?;
    
    if !result.success {
        return Err(anyhow::anyhow!("Wait condition was not met"));
    }
    
    Ok(())
}

async fn execute_wait_for_condition(
    url: &str,
    condition_type: &str,
    value: &str,
    selector: Option<&str>,
    count: Option<usize>,
    timeout: u64,
    interval: u64,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("🔍 Waiting for condition '{}' = '{}' on {}", condition_type, value, url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse condition type and create WaitCondition enum
    let wait_condition = match condition_type.to_lowercase().as_str() {
        "url-contains" => WaitCondition::UrlContains(value.to_string()),
        "url-equals" => WaitCondition::UrlEquals(value.to_string()),
        "title-contains" => WaitCondition::TitleContains(value.to_string()),
        "title-equals" => WaitCondition::TitleEquals(value.to_string()),
        "element-count" => {
            let selector = selector.ok_or_else(|| anyhow::anyhow!("--selector is required for element-count condition"))?;
            let count = count.ok_or_else(|| anyhow::anyhow!("--count is required for element-count condition"))?;
            WaitCondition::ElementCount { 
                selector: selector.to_string(), 
                count 
            }
        }
        "custom-js" => WaitCondition::CustomJs(value.to_string()),
        _ => {
            return Err(anyhow::anyhow!("Invalid condition type '{}'. Valid types: url-contains, url-equals, title-contains, title-equals, element-count, custom-js", condition_type));
        }
    };
    
    // Create wait strategy
    let wait_strategy = WaitStrategy {
        timeout_ms: timeout,
        poll_interval_ms: interval,
        ..Default::default()
    };
    
    // Create wait tool
    let wait_tool = WaitForCondition::new(Arc::new(browser.driver().clone()));
    
    // Create input
    let input = WaitForConditionInput {
        condition: wait_condition,
        strategy: Some(wait_strategy),
        js_args: None,
    };
    
    println!("⏳ Starting condition wait with timeout: {}ms, interval: {}ms", timeout, interval);
    
    // Execute the wait operation
    let result = wait_tool.execute(input).await?;
    
    if result.success && result.condition_met {
        println!("✅ Condition met successfully!");
        println!("   - Wait time: {}ms", result.wait_time_ms);
        println!("   - Attempts: {}", result.attempts);
        if let Some(final_value) = result.final_value {
            println!("   - Final value: {}", final_value);
        }
    } else {
        println!("❌ Wait condition not met:");
        println!("   - Wait time: {}ms", result.wait_time_ms);
        println!("   - Attempts: {}", result.attempts);
        if let Some(final_value) = result.final_value {
            println!("   - Last value: {}", final_value);
        }
        if let Some(error) = result.error_message {
            println!("   - Error: {}", error);
        }
    }
    
    // Take a screenshot to show the final state
    let screenshot_name = format!("wait_condition_result_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    browser.take_screenshot(&screenshot_name).await?;
    println!("📸 Screenshot saved: screenshots/{}", screenshot_name);
    
    // Record operation
    cost_tracker.record_operation(
        "wait_for_condition".to_string(),
        format!("Wait for condition '{}' = '{}' at {}", condition_type, value, url),
        0.007, // Slightly higher cost due to JavaScript execution
        result.success && result.condition_met,
    )?;
    
    // Close browser
    browser.close().await?;
    
    if !result.success || !result.condition_met {
        return Err(anyhow::anyhow!("Wait condition was not met"));
    }
    
    Ok(())
}

async fn execute_extract_text(
    url: &str,
    selector: Option<&str>,
    extraction_type: &str,
    format: &str,
    multiple: bool,
    include_metadata: bool,
    clean: bool,
    max_items: usize,
    min_length: Option<usize>,
    max_length: Option<usize>,
    filters: Option<&Vec<String>>,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("📄 Extracting text from {}", url);
    println!("   - Selector: {}", selector.unwrap_or("body"));
    println!("   - Extraction type: {}", extraction_type);
    println!("   - Format: {}", format);
    println!("   - Multiple elements: {}", multiple);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse extraction type
    let parsed_extraction_type = extraction_type.parse::<TextExtractionType>()
        .map_err(|e| anyhow::anyhow!("Invalid extraction type: {}", e))?;
    
    // Parse output format
    let parsed_format = format.parse::<OutputFormat>()
        .map_err(|e| anyhow::anyhow!("Invalid output format: {}", e))?;
    
    // Create extraction config
    let config = ExtractionConfig {
        format: parsed_format,
        scope: if let Some(sel) = selector {
            ExtractionScope::Element(sel.to_string())
        } else {
            ExtractionScope::Page
        },
        include_metadata,
        clean_text: clean,
        max_items,
        options: std::collections::HashMap::new(),
    };
    
    // Create extract text tool
    let extract_tool = ExtractText::new(Arc::new(browser.driver().clone()));
    
    // Create input
    let input = ExtractTextInput {
        selector: selector.map(|s| s.to_string()),
        extraction_type: parsed_extraction_type,
        config,
        extract_multiple: multiple,
        filters: filters.cloned(),
        min_length,
        max_length,
    };
    
    println!("🔍 Starting text extraction...");
    
    // Execute the extraction
    let result = extract_tool.execute(input).await?;
    
    println!("✅ Text extraction completed!");
    println!("   - Items extracted: {}", result.total_count);
    println!("   - Total text length: {} characters", result.total_length);
    
    // Output results
    if let Some(file_path) = output_file {
        // Write to file
        std::fs::write(file_path, &result.formatted_output)?;
        println!("📄 Output saved to: {}", file_path);
    } else {
        // Print to stdout
        println!("\n{}", result.formatted_output);
    }
    
    // Display individual items summary if multiple items
    if result.total_count > 1 {
        println!("\n📊 Extraction Summary:");
        for (i, item) in result.items.iter().enumerate() {
            println!("  {}. {} ({} chars) - {}", 
                     i + 1, 
                     item.tag_name, 
                     item.length,
                     if item.text.len() > 50 { 
                         format!("{}...", &item.text[..50]) 
                     } else { 
                         item.text.clone() 
                     });
        }
    }
    
    // Record operation
    cost_tracker.record_operation(
        "extract_text".to_string(),
        format!("Extract {} text from {} ({})", extraction_type, url, result.total_count),
        0.005, // Standard cost for text extraction
        result.total_count > 0,
    )?;
    
    // Close browser
    browser.close().await?;
    
    if result.total_count == 0 {
        return Err(anyhow::anyhow!("No text was extracted - check your selector and extraction type"));
    }
    
    Ok(())
}

async fn execute_extract_data(
    url: &str,
    template_file: Option<&str>,
    fields: Option<&Vec<String>>,
    root_selector: Option<&str>,
    multiple: bool,
    max_records: Option<usize>,
    format: &str,
    include_metadata: bool,
    validate: bool,
    skip_invalid: bool,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("📊 Extracting structured data from {}", url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse output format
    let parsed_format = format.parse::<OutputFormat>()
        .map_err(|e| anyhow::anyhow!("Invalid output format: {}", e))?;
    
    // Create extraction template
    let template = if let Some(template_path) = template_file {
        // Load template from file
        println!("📄 Loading template from: {}", template_path);
        load_template_from_file(template_path).await?
    } else if let Some(field_definitions) = fields {
        // Create template from field definitions
        println!("🏗️ Creating template from field definitions");
        create_template_from_fields(field_definitions, root_selector, multiple, max_records)?
    } else {
        // Default template - extract basic page information
        println!("🔧 Using default template for basic page data");
        create_default_template()
    };
    
    println!("   - Template: {} ({} fields)", template.name, template.fields.len());
    println!("   - Multiple records: {}", template.extract_multiple);
    println!("   - Format: {}", format);
    
    // Create extraction config
    let config = ExtractionConfig {
        format: parsed_format,
        scope: ExtractionScope::Page,
        include_metadata,
        clean_text: true,
        max_items: 0,
        options: std::collections::HashMap::new(),
    };
    
    // Create extract data tool
    let extract_tool = ExtractData::new(Arc::new(browser.driver().clone()));
    
    // Create input
    let input = ExtractDataInput {
        template,
        config,
        validate_data: validate,
        skip_invalid,
    };
    
    println!("🔍 Starting structured data extraction...");
    
    // Execute the extraction
    let result = extract_tool.execute(input).await?;
    
    println!("✅ Structured data extraction completed!");
    println!("   - Records extracted: {}", result.total_count);
    println!("   - Valid records: {}", result.valid_count);
    if result.invalid_count > 0 {
        println!("   - Invalid records: {}", result.invalid_count);
    }
    
    // Output results
    if let Some(file_path) = output_file {
        // Write to file
        std::fs::write(file_path, &result.formatted_output)?;
        println!("📄 Output saved to: {}", file_path);
    } else {
        // Print to stdout
        println!("\n{}", result.formatted_output);
    }
    
    // Display record summaries
    if result.total_count > 1 {
        println!("\n📊 Extraction Summary:");
        for (i, record) in result.records.iter().enumerate().take(5) { // Show first 5 records
            let status = if record.valid { "✅" } else { "❌" };
            let field_count = record.data.len();
            println!("  {}. {} {} fields - {}", 
                     i + 1, 
                     status,
                     field_count,
                     if record.validation_errors.is_empty() { 
                         "Valid".to_string() 
                     } else { 
                         format!("{} error(s)", record.validation_errors.len())
                     });
        }
        
        if result.total_count > 5 {
            println!("  ... and {} more records", result.total_count - 5);
        }
    }
    
    // Display validation errors for invalid records (if any)
    let invalid_records: Vec<_> = result.records.iter()
        .filter(|r| !r.valid)
        .take(3) // Show first 3 invalid records
        .collect();
    
    if !invalid_records.is_empty() {
        println!("\n⚠️  Validation Errors:");
        for (i, record) in invalid_records.iter().enumerate() {
            println!("  Record {}: ", record.index + 1);
            for error in &record.validation_errors {
                println!("    - {}", error);
            }
        }
    }
    
    // Record operation
    cost_tracker.record_operation(
        "extract_data".to_string(),
        format!("Extract structured data from {} ({} records)", url, result.total_count),
        0.008, // Higher cost due to advanced processing
        result.total_count > 0 && (result.valid_count > 0 || !validate),
    )?;
    
    // Close browser
    browser.close().await?;
    
    if result.total_count == 0 {
        return Err(anyhow::anyhow!("No data was extracted - check your template and selectors"));
    }
    
    if validate && result.valid_count == 0 {
        return Err(anyhow::anyhow!("No valid records found - check your data validation rules"));
    }
    
    Ok(())
}

async fn execute_extract_table(
    url: &str,
    table_selector: Option<&str>,
    multiple: bool,
    column_types: Option<&Vec<String>>,
    auto_infer: bool,
    include_headers: bool,
    include_footers: bool,
    skip_empty_rows: bool,
    min_columns: usize,
    max_rows: usize,
    column_mapping: Option<&Vec<String>>,
    include_columns: Option<&Vec<String>>,
    exclude_columns: Option<&Vec<String>>,
    format: &str,
    include_metadata: bool,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("📊 Extracting table data from {}", url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse output format
    let parsed_format = format.parse::<OutputFormat>()
        .map_err(|e| anyhow::anyhow!("Invalid output format: {}", e))?;
    
    // Parse column types
    let mut column_type_map = HashMap::new();
    if let Some(types) = column_types {
        for type_def in types {
            if let Some((column, data_type)) = type_def.split_once(':') {
                let parsed_type = data_type.parse::<TableDataType>()
                    .map_err(|e| anyhow::anyhow!("Invalid column type '{}': {}", data_type, e))?;
                column_type_map.insert(column.to_string(), parsed_type);
            }
        }
    }
    
    // Parse column mapping
    let mut mapping = HashMap::new();
    if let Some(mappings) = column_mapping {
        for mapping_def in mappings {
            if let Some((old_name, new_name)) = mapping_def.split_once(':') {
                mapping.insert(old_name.to_string(), new_name.to_string());
            }
        }
    }
    
    // Create table extraction config
    let table_config = TableExtractionConfig {
        include_headers,
        include_footers,
        merge_spanned_cells: true,
        skip_empty_rows,
        min_columns,
        max_rows,
        column_mapping: mapping,
        include_columns: include_columns.map(|v| v.clone()).unwrap_or_default(),
        exclude_columns: exclude_columns.map(|v| v.clone()).unwrap_or_default(),
    };
    
    // Create extraction config
    let config = ExtractionConfig {
        format: parsed_format,
        scope: ExtractionScope::Page,
        include_metadata,
        clean_text: true,
        max_items: 0,
        options: HashMap::new(),
    };
    
    // Create extract table tool
    let extract_tool = ExtractTable::new(Arc::new(browser.driver().clone()));
    
    // Create input
    let input = ExtractTableInput {
        table_selector: table_selector.map(|s| s.to_string()),
        config,
        table_config,
        extract_multiple: multiple,
        column_types: column_type_map,
        auto_infer_types: auto_infer,
    };
    
    println!("🔍 Starting table data extraction...");
    println!("   - Table selector: {}", table_selector.unwrap_or("table"));
    println!("   - Multiple tables: {}", multiple);
    println!("   - Auto-infer types: {}", auto_infer);
    println!("   - Format: {}", format);
    
    // Execute the extraction
    let result = extract_tool.execute(input).await?;
    
    println!("✅ Table extraction completed!");
    println!("   - Tables found: {}", result.table_count);
    println!("   - Total rows: {}", result.total_rows);
    
    // Display table summaries
    for (i, table) in result.tables.iter().enumerate() {
        println!("   Table {}: {} columns, {} rows", 
                 i + 1, 
                 table.structure.columns.len(), 
                 table.rows.len());
        
        if let Some(caption) = &table.structure.caption {
            println!("     Caption: {}", caption);
        }
        
        // Show column information
        if table.structure.columns.len() <= 10 {
            let column_names: Vec<&String> = table.structure.columns.iter()
                .map(|c| &c.name)
                .collect();
            println!("     Columns: {}", column_names.join(", "));
        } else {
            println!("     Columns: {} columns total", table.structure.columns.len());
        }
    }
    
    // Output results
    if let Some(file_path) = output_file {
        // Write to file
        std::fs::write(file_path, &result.formatted_output)?;
        println!("📄 Output saved to: {}", file_path);
    } else {
        // Print to stdout
        println!("\n{}", result.formatted_output);
    }
    
    // Record operation
    cost_tracker.record_operation(
        "extract_table".to_string(),
        format!("Extract table data from {} ({} tables, {} rows)", url, result.table_count, result.total_rows),
        0.010, // Higher cost due to complex table processing
        result.table_count > 0 && result.total_rows > 0,
    )?;
    
    // Close browser
    browser.close().await?;
    
    if result.table_count == 0 {
        return Err(anyhow::anyhow!("No tables found - check your table selector"));
    }
    
    if result.total_rows == 0 {
        return Err(anyhow::anyhow!("No table data extracted - tables may be empty or below minimum column requirement"));
    }
    
    Ok(())
}

/// Load extraction template from file
async fn load_template_from_file(file_path: &str) -> Result<ExtractionTemplate> {
    let content = std::fs::read_to_string(file_path)?;
    
    // Try JSON first, then YAML
    if file_path.ends_with(".json") {
        serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON template: {}", e))
    } else if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML template: {}", e))
    } else {
        // Try JSON first, fallback to YAML
        serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))
            .map_err(|e| anyhow::anyhow!("Failed to parse template (tried JSON and YAML): {}", e))
    }
}

/// Create extraction template from field definitions
fn create_template_from_fields(
    field_definitions: &[String], 
    root_selector: Option<&str>,
    multiple: bool,
    max_records: Option<usize>
) -> Result<ExtractionTemplate> {
    let mut fields = Vec::new();
    
    for (index, field_def) in field_definitions.iter().enumerate() {
        let parts: Vec<&str> = field_def.split(':').collect();
        
        let (name, selector_str, data_type_str) = match parts.len() {
            2 => (parts[0], parts[1], "string"), // name:selector (default to string)
            3 => (parts[0], parts[1], parts[2]), // name:selector:type
            _ => return Err(anyhow::anyhow!("Invalid field definition '{}'. Use format 'name:selector' or 'name:selector:type'", field_def))
        };
        
        let selector = selector_str.parse::<SelectorType>()
            .map_err(|e| anyhow::anyhow!("Invalid selector in field '{}': {}", name, e))?;
        
        let data_type = data_type_str.parse::<DataType>()
            .map_err(|e| anyhow::anyhow!("Invalid data type in field '{}': {}", name, e))?;
        
        fields.push(DataField {
            name: name.to_string(),
            selector,
            data_type,
            attribute: None,
            required: false,
            default_value: None,
            validation_pattern: None,
            transform_function: None,
        });
    }
    
    let root_selector_type = if let Some(root_sel) = root_selector {
        Some(root_sel.parse::<SelectorType>()
            .map_err(|e| anyhow::anyhow!("Invalid root selector: {}", e))?)
    } else {
        None
    };
    
    Ok(ExtractionTemplate {
        name: "CLI Template".to_string(),
        description: Some("Template created from CLI field definitions".to_string()),
        fields,
        root_selector: root_selector_type,
        extract_multiple: multiple,
        max_records,
    })
}

/// Create default template for basic page information
fn create_default_template() -> ExtractionTemplate {
    let fields = vec![
        DataField {
            name: "title".to_string(),
            selector: SelectorType::Css("title".to_string()),
            data_type: DataType::String,
            attribute: None,
            required: false,
            default_value: None,
            validation_pattern: None,
            transform_function: None,
        },
        DataField {
            name: "description".to_string(),
            selector: SelectorType::Css("meta[name='description']".to_string()),
            data_type: DataType::String,
            attribute: Some("content".to_string()),
            required: false,
            default_value: None,
            validation_pattern: None,
            transform_function: None,
        },
        DataField {
            name: "heading".to_string(),
            selector: SelectorType::Css("h1".to_string()),
            data_type: DataType::String,
            attribute: None,
            required: false,
            default_value: None,
            validation_pattern: None,
            transform_function: None,
        },
    ];
    
    ExtractionTemplate {
        name: "Default Page Data".to_string(),
        description: Some("Extract basic page information (title, description, heading)".to_string()),
        fields,
        root_selector: None,
        extract_multiple: false,
        max_records: None,
    }
}

async fn execute_extract_form(
    url: &str,
    form_selector: Option<&str>,
    multiple: bool,
    include_hidden: bool,
    include_disabled: bool,
    extract_options: bool,
    analyze_validation: bool,
    include_positioning: bool,
    max_options_per_field: usize,
    format: &str,
    include_metadata: bool,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("📝 Extracting form data from {}", url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse output format
    let parsed_format = format.parse::<OutputFormat>()
        .map_err(|e| anyhow::anyhow!("Invalid output format: {}", e))?;
    
    // Create form extraction configuration
    let form_config = FormExtractionConfig {
        include_hidden,
        include_disabled,
        extract_options,
        analyze_validation,
        include_positioning,
        max_options_per_field,
    };
    
    let config = ExtractionConfig {
        format: parsed_format,
        scope: ExtractionScope::Page,
        include_metadata,
        clean_text: true,
        max_items: 0,
        options: HashMap::new(),
    };
    
    let input = ExtractFormInput {
        form_selector: form_selector.map(|s| s.to_string()),
        config: config.clone(),
        form_config: form_config.clone(),
        extract_multiple: multiple,
    };
    
    // Create form extraction tool
    let driver = Arc::new(browser.driver.clone());
    let extract_form_tool = ExtractForm::new(driver);
    
    // Record API cost before extraction
    cost_tracker.record_browser_operation_cost();
    
    // Execute form extraction
    let start_time = std::time::Instant::now();
    let result = extract_form_tool.execute(input).await
        .context("Failed to extract forms from page")?;
    let duration = start_time.elapsed();
    
    // Display results summary
    println!("📊 Form Extraction Summary:");
    println!("   Forms found: {}", result.form_count);
    println!("   Total fields: {}", result.total_fields);
    println!("   Duration: {:.2}s", duration.as_secs_f64());
    
    if result.form_count > 0 {
        println!("\n📋 Form Details:");
        for (i, form) in result.forms.iter().enumerate() {
            println!("   Form {}: {} fields", i + 1, form.fields.len());
            if let Some(action) = &form.action {
                println!("     Action: {}", action);
            }
            println!("     Method: {}", form.method);
            if form.has_validation {
                println!("     Has validation: Yes");
            }
            println!("     Submit buttons: {}", form.submit_buttons.len());
        }
    }
    
    // Save or display output
    if let Some(file_path) = output_file {
        std::fs::write(file_path, &result.formatted_output)
            .context("Failed to save form data to file")?;
        println!("📄 Form data saved to: {}", file_path);
    } else {
        println!("\n📝 Extracted Form Data:");
        println!("{}", result.formatted_output);
    }
    
    Ok(())
}

async fn execute_extract_links(
    url: &str,
    link_selector: Option<&str>,
    include_anchors: bool,
    include_email: bool,
    include_phone: bool,
    include_javascript: bool,
    validate_urls: bool,
    categorize_links: bool,
    include_positioning: bool,
    analyze_accessibility: bool,
    min_text_length: usize,
    max_links: usize,
    internal_domains: Option<&Vec<String>>,
    exclude_patterns: Option<&Vec<String>>,
    format: &str,
    include_metadata: bool,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("🔗 Extracting links from {}", url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse output format
    let parsed_format = format.parse::<OutputFormat>()
        .map_err(|e| anyhow::anyhow!("Invalid output format: {}", e))?;
    
    // Create link extraction configuration
    let link_config = LinkExtractionConfig {
        include_anchors,
        include_email,
        include_phone,
        include_javascript,
        validate_urls,
        categorize_links,
        include_positioning,
        analyze_accessibility,
        min_text_length,
        max_links,
        internal_domains: internal_domains.cloned().unwrap_or_default(),
        exclude_patterns: exclude_patterns.cloned().unwrap_or_default(),
    };
    
    let config = ExtractionConfig {
        format: parsed_format,
        scope: ExtractionScope::Page,
        include_metadata,
        clean_text: true,
        max_items: 0,
        options: HashMap::new(),
    };
    
    let input = ExtractLinksInput {
        link_selector: link_selector.map(|s| s.to_string()),
        config: config.clone(),
        link_config: link_config.clone(),
    };
    
    // Create link extraction tool
    let driver = Arc::new(browser.driver.clone());
    let extract_links_tool = ExtractLinks::new(driver);
    
    // Record API cost before extraction
    cost_tracker.record_browser_operation_cost();
    
    // Execute link extraction
    let start_time = std::time::Instant::now();
    let result = extract_links_tool.execute(input).await
        .context("Failed to extract links from page")?;
    let duration = start_time.elapsed();
    
    // Display results summary
    println!("📊 Link Extraction Summary:");
    println!("   Links found: {}", result.link_count);
    println!("   Internal links: {}", result.statistics.internal_count);
    println!("   External links: {}", result.statistics.external_count);
    println!("   Valid URLs: {}", result.statistics.valid_urls);
    println!("   Invalid URLs: {}", result.statistics.invalid_urls);
    println!("   Accessible links: {}", result.statistics.accessible_links);
    println!("   Duration: {:.2}s", duration.as_secs_f64());
    
    if !result.statistics.by_type.is_empty() {
        println!("\n📋 Links by Type:");
        for (link_type, count) in &result.statistics.by_type {
            println!("   {}: {}", link_type, count);
        }
    }
    
    if !result.statistics.unique_domains.is_empty() {
        println!("\n🌐 Unique Domains ({}):", result.statistics.unique_domains.len());
        for (i, domain) in result.statistics.unique_domains.iter().enumerate() {
            if i < 10 {  // Show first 10 domains
                println!("   {}", domain);
            } else if i == 10 {
                println!("   ... and {} more", result.statistics.unique_domains.len() - 10);
                break;
            }
        }
    }
    
    // Save or display output
    if let Some(file_path) = output_file {
        std::fs::write(file_path, &result.formatted_output)
            .context("Failed to save link data to file")?;
        println!("📄 Link data saved to: {}", file_path);
    } else {
        println!("\n🔗 Extracted Link Data:");
        println!("{}", result.formatted_output);
    }
    
    Ok(())
}

async fn execute_smart_actions(
    url: &str,
    form_selector: Option<&str>,
    form_data: Option<&Vec<String>>,
    user_preferences: Option<&Vec<String>>,
    field_mappings: Option<&Vec<String>>,
    intelligent_detection: bool,
    smart_suggestions: bool,
    validate_before_fill: bool,
    wait_for_visibility: bool,
    max_wait_time: u64,
    retry_attempts: usize,
    auto_submit: bool,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    use std::sync::Arc;
    
    println!("🤖 Executing smart actions on {}", url);
    
    let mut browser = SimpleBrowser::new().await?;
    browser.navigate(url, true).await?;
    
    // Parse form data from key=value pairs
    let mut parsed_form_data = HashMap::new();
    if let Some(data_pairs) = form_data {
        for pair in data_pairs {
            if let Some((key, value)) = pair.split_once('=') {
                parsed_form_data.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    // Parse user preferences from key=value pairs
    let mut parsed_preferences = HashMap::new();
    if let Some(pref_pairs) = user_preferences {
        for pair in pref_pairs {
            if let Some((key, value)) = pair.split_once('=') {
                parsed_preferences.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    // Parse field mappings from old:new pairs
    let mut parsed_mappings = HashMap::new();
    if let Some(mapping_pairs) = field_mappings {
        for pair in mapping_pairs {
            if let Some((old_name, new_name)) = pair.split_once(':') {
                parsed_mappings.insert(old_name.to_string(), new_name.to_string());
            }
        }
    }
    
    // Create smart action configuration
    let config = SmartActionConfig {
        use_intelligent_detection: intelligent_detection,
        validate_before_fill,
        wait_for_visibility,
        max_wait_time,
        use_smart_suggestions: smart_suggestions,
        handle_dynamic_content: true,
        retry_attempts,
    };
    
    let input = SmartActionsInput {
        form_selector: form_selector.map(|s| s.to_string()),
        form_data: parsed_form_data,
        user_preferences: parsed_preferences,
        config: config.clone(),
        auto_submit,
        field_mappings: parsed_mappings,
    };
    
    // Create smart actions tool
    let driver = Arc::new(browser.driver.clone());
    let mut smart_actions_tool = SmartActions::new(driver);
    
    // Record API cost before execution
    cost_tracker.record_browser_operation_cost();
    
    // Execute smart actions
    let start_time = std::time::Instant::now();
    let result = smart_actions_tool.execute(input).await
        .context("Failed to execute smart actions")?;
    let duration = start_time.elapsed();
    
    // Display results summary
    println!("🤖 Smart Actions Summary:");
    println!("   Fields processed: {}", result.fields_processed.len());
    println!("   Actions executed: {}", result.actions_executed.len());
    println!("   Success rate: {:.1}%", result.success_rate * 100.0);
    println!("   Duration: {:.2}s", duration.as_secs_f64());
    
    if !result.fields_processed.is_empty() {
        println!("\n📋 Field Analysis:");
        for field in &result.fields_processed {
            println!("   {}: {:?} (confidence: {:.1}%)", 
                field.name, field.field_type, field.confidence * 100.0);
            if let Some(suggested) = &field.suggested_value {
                println!("     Suggested value: {}", suggested);
            }
        }
    }
    
    if !result.actions_executed.is_empty() {
        println!("\n⚡ Actions Performed:");
        for action in &result.actions_executed {
            let status = if action.success { "✅" } else { "❌" };
            println!("   {} {:?} on {} ({}ms)", 
                status, action.action_type, 
                action.target_selector.as_deref().unwrap_or("unknown"),
                action.duration_ms);
            if let Some(error) = &action.error_message {
                println!("     Error: {}", error);
            }
        }
    }
    
    if !result.improvement_suggestions.is_empty() {
        println!("\n💡 Improvement Suggestions:");
        for suggestion in &result.improvement_suggestions {
            println!("   • {}", suggestion);
        }
    }
    
    // Save or display detailed output
    if let Some(file_path) = output_file {
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize smart actions result")?;
        std::fs::write(file_path, json_output)
            .context("Failed to save smart actions result to file")?;
        println!("📄 Smart actions result saved to: {}", file_path);
    } else {
        println!("\n🤖 Detailed Results:");
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize smart actions result")?;
        println!("{}", json_output);
    }
    
    Ok(())
}

/// Execute workflow orchestrator command
async fn execute_workflow_orchestrator(
    url: &str,
    workflow_file: &str,
    variables: Option<&Vec<String>>,
    resume_state: Option<&str>,
    capture_screenshots: bool,
    save_state: bool,
    enable_parallel: bool,
    strict_validation: bool,
    max_parallel: usize,
    step_delay_ms: u64,
    detailed_logging: bool,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    let start_time = std::time::Instant::now();
    info!("🤖 Starting workflow orchestrator for URL: {}", url);
    
    // Parse variables
    let mut parsed_variables = HashMap::new();
    if let Some(vars) = variables {
        for var_pair in vars {
            if let Some((key, value)) = var_pair.split_once('=') {
                parsed_variables.insert(key.to_string(), serde_json::Value::String(value.to_string()));
            } else {
                warn!("Invalid variable format: {}. Expected key=value", var_pair);
            }
        }
    }
    
    // Load workflow definition from file
    let workflow_content = std::fs::read_to_string(workflow_file)
        .with_context(|| format!("Failed to read workflow file: {}", workflow_file))?;
    
    let workflow: WorkflowDefinition = serde_json::from_str(&workflow_content)
        .with_context(|| format!("Failed to parse workflow JSON from file: {}", workflow_file))?;
    
    info!("📋 Loaded workflow '{}' with {} steps", workflow.name, workflow.steps.len());
    
    // Load resume state if specified
    let resume_workflow_state = if let Some(state_file) = resume_state {
        let state_content = std::fs::read_to_string(state_file)
            .with_context(|| format!("Failed to read resume state file: {}", state_file))?;
        let state = serde_json::from_str(&state_content)
            .with_context(|| format!("Failed to parse workflow state from file: {}", state_file))?;
        Some(state)
    } else {
        None
    };
    
    // Create browser and navigate to initial URL
    let browser = SimpleBrowser::new().await?;
    let driver = browser.create_driver().await?;
    
    info!("🌐 Navigating to: {}", url);
    driver.goto(url).await?;
    
    let mut workflow_tool = WorkflowOrchestrator::new(driver);
    
    let config = WorkflowConfig {
        capture_screenshots,
        save_state,
        enable_parallel,
        strict_validation,
        max_parallel,
        step_delay_ms,
        detailed_logging,
    };
    
    // Merge global variables with provided variables
    let mut initial_variables = workflow.global_variables.clone();
    initial_variables.extend(parsed_variables);
    
    let input = WorkflowOrchestratorInput {
        workflow,
        initial_variables,
        config,
        resume_from_state: resume_workflow_state,
    };
    
    // Execute the workflow
    info!("🚀 Executing workflow with {} steps", input.workflow.steps.len());
    let result = workflow_tool.execute(input).await
        .context("Failed to execute workflow orchestrator")?;
    
    let execution_time = start_time.elapsed();
    
    // Update cost tracker
    cost_tracker.track_automation_cost(1, execution_time);
    
    // Display results
    println!("\n🎯 Workflow Orchestration Complete!");
    println!("⏱️  Total execution time: {:.2}s", execution_time.as_secs_f64());
    println!("📊 Success rate: {:.1}%", result.success_rate * 100.0);
    println!("🔢 Total actions executed: {}", result.executed_actions.len());
    
    // Show final workflow state
    println!("\n📋 Final Workflow State:");
    println!("   Status: {}", if result.final_state.is_complete {
        "✅ Complete"
    } else if result.final_state.has_failed {
        "❌ Failed"
    } else {
        "⏸️ Incomplete"
    });
    
    if let Some(error) = &result.final_state.error_message {
        println!("   Error: {}", error);
    }
    
    println!("   Steps executed: {}", result.final_state.execution_history.len());
    println!("   Duration: {}ms", result.final_state.total_duration_ms);
    
    // Show execution history summary
    if !result.final_state.execution_history.is_empty() {
        println!("\n📈 Execution History:");
        for (i, execution) in result.final_state.execution_history.iter().enumerate() {
            let status = if execution.success { "✅" } else { "❌" };
            println!("   {}. {} {} - {} ({}ms)", 
                i + 1,
                status,
                execution.step_name,
                if execution.success { "Success" } else { "Failed" },
                execution.duration_ms
            );
            if let Some(error) = &execution.error_message {
                println!("      Error: {}", error);
            }
        }
    }
    
    // Show performance metrics
    println!("\n⚡ Performance Metrics:");
    println!("   Average action time: {}ms", result.metrics.total_duration_ms / result.metrics.actions_count.max(1) as u64);
    println!("   Failed actions: {}", result.metrics.failed_actions);
    if !result.metrics.page_loads.is_empty() {
        let avg_load = result.metrics.page_loads.iter().sum::<u64>() / result.metrics.page_loads.len() as u64;
        println!("   Average page load: {}ms", avg_load);
    }
    
    // Show screenshots if captured
    if !result.screenshots.is_empty() {
        println!("\n📸 Screenshots captured: {}", result.screenshots.len());
        for screenshot in &result.screenshots {
            println!("   • {}: {}", screenshot.step_id, screenshot.description);
        }
    }
    
    // Show recommendations
    if !result.recommendations.is_empty() {
        println!("\n💡 Workflow Recommendations:");
        for recommendation in &result.recommendations {
            println!("   • {}", recommendation);
        }
    }
    
    // Save or display detailed output
    if let Some(file_path) = output_file {
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize workflow result")?;
        std::fs::write(file_path, json_output)
            .context("Failed to save workflow result to file")?;
        println!("📄 Workflow result saved to: {}", file_path);
    } else {
        println!("\n🤖 Detailed Results:");
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize workflow result")?;
        println!("{}", json_output);
    }
    
    Ok(())
}

/// Execute visual validator command
async fn execute_visual_validator(
    url: &str,
    test_types: Option<&Vec<String>>,
    baseline_path: Option<&str>,
    output_directory: &str,
    target_elements: Option<&Vec<String>>,
    viewport_sizes: Option<&Vec<String>>,
    similarity_threshold: f64,
    pixel_tolerance: u32,
    color_tolerance: f64,
    anti_aliasing_tolerance: bool,
    filter_dynamic_content: bool,
    generate_reports: bool,
    save_differences: bool,
    image_quality: u8,
    output_file: Option<&str>,
    cost_tracker: &mut CostTracker
) -> Result<()> {
    let start_time = std::time::Instant::now();
    info!("🎨 Starting visual validation for URL: {}", url);
    
    // Parse test types
    let parsed_test_types = if let Some(types) = test_types {
        let mut parsed = Vec::new();
        for type_str in types {
            let test_type = match type_str.as_str() {
                "screenshot-comparison" | "screenshot_comparison" => VisualTestType::ScreenshotComparison,
                "element-validation" | "element_validation" => VisualTestType::ElementValidation,
                "layout-validation" | "layout_validation" => VisualTestType::LayoutValidation,
                "color-validation" | "color_validation" => VisualTestType::ColorValidation,
                "typography-validation" | "typography_validation" => VisualTestType::TypographyValidation,
                "responsive-validation" | "responsive_validation" => VisualTestType::ResponsiveValidation,
                "accessibility-validation" | "accessibility_validation" => VisualTestType::AccessibilityValidation,
                "regression-testing" | "regression_testing" => VisualTestType::RegressionTesting,
                "cross-browser-validation" | "cross_browser_validation" => VisualTestType::CrossBrowserValidation,
                _ => {
                    warn!("Unknown test type: {}. Using screenshot comparison", type_str);
                    VisualTestType::ScreenshotComparison
                }
            };
            parsed.push(test_type);
        }
        parsed
    } else {
        vec![VisualTestType::ScreenshotComparison]
    };
    
    // Parse viewport sizes
    let parsed_viewport_sizes = if let Some(sizes) = viewport_sizes {
        let mut parsed = Vec::new();
        for size_str in sizes {
            if let Some((width_str, height_str)) = size_str.split_once('x') {
                if let (Ok(width), Ok(height)) = (width_str.parse::<u32>(), height_str.parse::<u32>()) {
                    parsed.push(ViewportSize { width, height });
                } else {
                    warn!("Invalid viewport size format: {}. Expected WIDTHxHEIGHT", size_str);
                }
            } else {
                warn!("Invalid viewport size format: {}. Expected WIDTHxHEIGHT", size_str);
            }
        }
        if parsed.is_empty() {
            // Default viewport sizes
            vec![
                ViewportSize { width: 1920, height: 1080 },
                ViewportSize { width: 1366, height: 768 },
                ViewportSize { width: 375, height: 667 },
            ]
        } else {
            parsed
        }
    } else {
        vec![
            ViewportSize { width: 1920, height: 1080 },
            ViewportSize { width: 1366, height: 768 },
            ViewportSize { width: 375, height: 667 },
        ]
    };
    
    // Create browser and navigate to URL
    let browser = SimpleBrowser::new().await?;
    let driver = browser.create_driver().await?;
    
    info!("🌐 Navigating to: {}", url);
    driver.goto(url).await?;
    
    let mut visual_validator = VisualValidator::new(driver);
    
    let config = VisualValidationConfig {
        similarity_threshold,
        pixel_tolerance,
        color_tolerance,
        anti_aliasing_tolerance,
        ignore_areas: Vec::new(),
        focus_areas: Vec::new(),
        filter_dynamic_content,
        screenshot_format: rainbow_poc::tools::advanced_automation::ImageFormat::Png,
        image_quality,
    };
    
    let input = VisualValidatorInput {
        test_types: parsed_test_types,
        baseline_path: baseline_path.map(|s| s.to_string()),
        output_directory: output_directory.to_string(),
        target_elements: target_elements.map(|v| v.clone()).unwrap_or_default(),
        viewport_sizes: parsed_viewport_sizes,
        config,
        generate_reports,
        save_differences,
    };
    
    // Execute visual validation
    info!("🔍 Executing visual validation with {} test types", input.test_types.len());
    let result = visual_validator.execute(input).await
        .context("Failed to execute visual validator")?;
    
    let execution_time = start_time.elapsed();
    
    // Update cost tracker
    cost_tracker.track_automation_cost(1, execution_time);
    
    // Display results
    println!("\n🎨 Visual Validation Complete!");
    println!("⏱️  Total execution time: {:.2}s", execution_time.as_secs_f64());
    println!("📊 Overall status: {}", if result.overall_passed { "✅ PASSED" } else { "❌ FAILED" });
    println!("🎯 Overall confidence: {:.1}%", result.overall_confidence * 100.0);
    
    // Show summary metrics
    println!("\n📈 Summary Metrics:");
    println!("   Tests performed: {}", result.summary_metrics.total_tests);
    println!("   Tests passed: {} / {}", result.summary_metrics.tests_passed, result.summary_metrics.total_tests);
    println!("   Average similarity: {:.1}%", result.summary_metrics.average_similarity);
    
    if result.summary_metrics.critical_findings > 0 {
        println!("   🚨 Critical findings: {}", result.summary_metrics.critical_findings);
    }
    if result.summary_metrics.high_findings > 0 {
        println!("   ⚠️  High priority findings: {}", result.summary_metrics.high_findings);
    }
    
    // Show validation results by test type
    if !result.validation_results.is_empty() {
        println!("\n📋 Test Results:");
        for validation in &result.validation_results {
            let status = if validation.passed { "✅" } else { "❌" };
            println!("   {} {:?} - {:.1}% confidence", 
                status,
                validation.test_type,
                validation.confidence * 100.0
            );
            
            if !validation.findings.is_empty() {
                for finding in &validation.findings {
                    let severity_icon = match finding.severity {
                        rainbow_poc::tools::advanced_automation::Severity::Critical => "🚨",
                        rainbow_poc::tools::advanced_automation::Severity::High => "⚠️ ",
                        rainbow_poc::tools::advanced_automation::Severity::Medium => "⚡",
                        rainbow_poc::tools::advanced_automation::Severity::Low => "💡",
                        rainbow_poc::tools::advanced_automation::Severity::Info => "ℹ️ ",
                    };
                    println!("      {} {}", severity_icon, finding.description);
                }
            }
        }
    }
    
    // Show screenshots captured
    if !result.screenshots.is_empty() {
        println!("\n📸 Screenshots Captured:");
        for screenshot in &result.screenshots {
            println!("   • {} ({}x{}) - {}", 
                screenshot.path,
                screenshot.viewport.width,
                screenshot.viewport.height,
                screenshot.description
            );
        }
    }
    
    // Show generated reports
    if !result.reports.is_empty() {
        println!("\n📄 Reports Generated:");
        for report in &result.reports {
            println!("   • {:?} report: {} ({} bytes)", 
                report.report_type,
                report.path,
                report.file_size
            );
        }
    }
    
    // Show recommendations
    if !result.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for recommendation in &result.recommendations {
            println!("   • {}", recommendation);
        }
    }
    
    // Show performance metrics
    println!("\n⚡ Performance Metrics:");
    println!("   Total pixels compared: {}", result.summary_metrics.total_pixels_compared);
    println!("   Processing time: {}ms", result.summary_metrics.total_processing_time_ms);
    
    // Save or display detailed output
    if let Some(file_path) = output_file {
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize visual validation result")?;
        std::fs::write(file_path, json_output)
            .context("Failed to save visual validation result to file")?;
        println!("📄 Visual validation result saved to: {}", file_path);
    } else {
        println!("\n🤖 Detailed Results:");
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize visual validation result")?;
        println!("{}", json_output);
    }
    
    Ok(())
}

/// Execute performance monitoring and analysis
async fn execute_performance_monitor(
    url: Option<&str>,
    iterations: usize,
    wait_after_load: u64,
    interval: u64,
    capture_core_web_vitals: bool,
    capture_resource_timing: bool,
    capture_memory_metrics: bool,
    capture_network_metrics: bool,
    generate_report: bool,
    output: Option<&str>,
    cost_tracker: &mut CostTracker,
) -> Result<()> {
    use rainbow_poc::tools::advanced_automation::{PerformanceRating, Severity, BottleneckType};
    
    info!("🎯 Starting performance monitor...");
    println!("🎯 Starting performance monitor");
    
    // Track operation cost
    cost_tracker.record_operation("performance_monitor", 1, 0);
    
    // Initialize browser with ChromeDriver manager
    let driver_manager = ChromeDriverManager::new().await?;
    driver_manager.ensure_driver().await?;
    let driver = driver_manager.create_driver().await
        .context("Failed to create WebDriver")?;
    
    // Navigate to URL if provided
    if let Some(url) = url {
        info!("📱 Navigating to: {}", url);
        println!("📱 Navigating to: {}", url);
        driver.goto(url).await
            .context("Failed to navigate to URL")?;
    } else {
        info!("📱 Using current page for performance monitoring");
        println!("📱 Using current page for performance monitoring");
    }
    
    // Create performance monitor
    let mut monitor = PerformanceMonitor::new(driver);
    
    // Configure monitoring
    let config = PerformanceConfig {
        wait_after_load_ms: wait_after_load,
        interval_ms: interval,
        capture_core_web_vitals,
        capture_resource_timing,
        capture_memory_metrics,
        capture_network_metrics,
        generate_report,
    };
    
    let input = PerformanceMonitorInput {
        url: url.map(|s| s.to_string()),
        iterations: Some(iterations),
        config,
    };
    
    // Execute performance monitoring
    info!("📊 Collecting performance metrics ({} iterations)...", iterations);
    println!("📊 Collecting performance metrics ({} iterations)...", iterations);
    
    let result = monitor.execute(input).await
        .context("Performance monitoring failed")?;
    
    // Display results
    println!("\n✅ Performance Monitoring Complete!");
    println!("⏱️  Monitoring Duration: {:.2}s", result.monitoring_duration_ms as f64 / 1000.0);
    
    // Display performance score
    println!("\n🏆 Performance Score:");
    let score_icon = match result.performance_score.rating {
        PerformanceRating::Good => "🟢",
        PerformanceRating::NeedsImprovement => "🟡",
        PerformanceRating::Poor => "🔴",
        PerformanceRating::NotAvailable => "⚪",
    };
    println!("  {} Overall Score: {:.1}/100", score_icon, result.performance_score.overall_score);
    
    // Display Core Web Vitals
    if let Some(latest_metrics) = result.metrics_history.last() {
        println!("\n📈 Core Web Vitals:");
        
        // LCP
        let lcp_icon = match latest_metrics.core_web_vitals.lcp_rating {
            PerformanceRating::Good => "🟢",
            PerformanceRating::NeedsImprovement => "🟡",
            PerformanceRating::Poor => "🔴",
            PerformanceRating::NotAvailable => "⚪",
        };
        println!("  {} Largest Contentful Paint: {:.2}s", 
            lcp_icon, 
            latest_metrics.core_web_vitals.largest_contentful_paint / 1000.0);
        
        // FID
        if let Some(fid) = latest_metrics.core_web_vitals.first_input_delay {
            let fid_icon = match latest_metrics.core_web_vitals.fid_rating {
                PerformanceRating::Good => "🟢",
                PerformanceRating::NeedsImprovement => "🟡",
                PerformanceRating::Poor => "🔴",
                PerformanceRating::NotAvailable => "⚪",
            };
            println!("  {} First Input Delay: {:.0}ms", fid_icon, fid);
        }
        
        // CLS
        let cls_icon = match latest_metrics.core_web_vitals.cls_rating {
            PerformanceRating::Good => "🟢",
            PerformanceRating::NeedsImprovement => "🟡",
            PerformanceRating::Poor => "🔴",
            PerformanceRating::NotAvailable => "⚪",
        };
        println!("  {} Cumulative Layout Shift: {:.3}", 
            cls_icon, 
            latest_metrics.core_web_vitals.cumulative_layout_shift);
        
        // Page load metrics
        println!("\n⚡ Page Load Metrics:");
        println!("  • Page Load Time: {:.2}s", latest_metrics.page_load_time / 1000.0);
        println!("  • DOM Content Loaded: {:.2}s", latest_metrics.dom_content_loaded / 1000.0);
        println!("  • Time to First Byte: {:.0}ms", latest_metrics.time_to_first_byte);
        
        if let Some(fp) = latest_metrics.first_paint {
            println!("  • First Paint: {:.2}s", fp / 1000.0);
        }
        if let Some(fcp) = latest_metrics.first_contentful_paint {
            println!("  • First Contentful Paint: {:.2}s", fcp / 1000.0);
        }
        
        // Resource metrics
        if capture_resource_timing {
            println!("\n📦 Resource Metrics:");
            println!("  • Total Resources: {}", latest_metrics.resource_metrics.total_resources);
            println!("  • Total Transfer Size: {:.2}MB", 
                latest_metrics.resource_metrics.total_transfer_size as f64 / 1_000_000.0);
            println!("  • Average Resource Duration: {:.0}ms", 
                latest_metrics.resource_metrics.average_duration);
            
            if let Some(slowest) = &latest_metrics.resource_metrics.slowest_resource {
                println!("  • Slowest Resource: {}", 
                    if slowest.len() > 50 { 
                        format!("{}...", &slowest[..50]) 
                    } else { 
                        slowest.clone() 
                    });
            }
        }
        
        // Memory metrics
        if capture_memory_metrics && latest_metrics.memory_metrics.used_js_heap_size > 0 {
            println!("\n💾 Memory Metrics:");
            println!("  • JS Heap Used: {:.2}MB", 
                latest_metrics.memory_metrics.used_js_heap_size as f64 / 1_000_000.0);
            println!("  • JS Heap Total: {:.2}MB", 
                latest_metrics.memory_metrics.total_js_heap_size as f64 / 1_000_000.0);
        }
    }
    
    // Display bottlenecks
    if !result.bottlenecks.is_empty() {
        println!("\n⚠️  Performance Bottlenecks:");
        for bottleneck in &result.bottlenecks {
            let severity_icon = match bottleneck.severity {
                Severity::Critical => "🔴",
                Severity::High => "🟠",
                Severity::Medium => "🟡",
                Severity::Low => "🟢",
            };
            println!("  {} {:?}: {}", severity_icon, bottleneck.bottleneck_type, bottleneck.impact);
            println!("    → {}", bottleneck.recommendation);
        }
    }
    
    // Display recommendations
    if !result.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for recommendation in &result.recommendations {
            println!("  • {}", recommendation);
        }
    }
    
    // Display aggregated metrics for multiple iterations
    if iterations > 1 {
        println!("\n📊 Aggregated Metrics ({} iterations):", iterations);
        println!("  • Average Page Load: {:.2}s", result.aggregated_metrics.average_page_load / 1000.0);
        println!("  • Average LCP: {:.2}s", result.aggregated_metrics.average_lcp / 1000.0);
        println!("  • Average CLS: {:.3}", result.aggregated_metrics.average_cls);
        if let Some(avg_fid) = result.aggregated_metrics.average_fid {
            println!("  • Average FID: {:.0}ms", avg_fid);
        }
    }
    
    // Display detailed report summary if generated
    if let Some(report) = &result.report {
        println!("\n📋 Performance Report Summary:");
        println!("  • Total Measurements: {}", report.summary.total_measurements);
        println!("  • P95 Page Load: {:.2}s", report.summary.p95_page_load / 1000.0);
        println!("  • P95 LCP: {:.2}s", report.summary.p95_lcp / 1000.0);
        
        // Trend analysis
        if report.trend_analysis.page_load_trend != 0.0 {
            println!("\n📈 Performance Trends:");
            let trend_icon = if report.trend_analysis.page_load_trend < 0.0 { "↓" } else { "↑" };
            println!("  • Page Load: {}{:.1}%", trend_icon, report.trend_analysis.page_load_trend.abs());
            
            let lcp_icon = if report.trend_analysis.lcp_trend < 0.0 { "↓" } else { "↑" };
            println!("  • LCP: {}{:.1}%", lcp_icon, report.trend_analysis.lcp_trend.abs());
            
            let cls_icon = if report.trend_analysis.cls_trend < 0.0 { "↓" } else { "↑" };
            println!("  • CLS: {}{:.1}%", cls_icon, report.trend_analysis.cls_trend.abs());
        }
    }
    
    // Save results to file if specified
    if let Some(file_path) = output {
        let json_data = serde_json::to_string_pretty(&result)
            .context("Failed to serialize performance results")?;
        std::fs::write(file_path, json_data)
            .context("Failed to save performance results to file")?;
        println!("\n📄 Performance results saved to: {}", file_path);
    } else if generate_report {
        println!("\n🤖 Full Performance Report:");
        let json_output = serde_json::to_string_pretty(&result)
            .context("Failed to serialize performance results")?;
        println!("{}", json_output);
    }
    
    Ok(())
}
