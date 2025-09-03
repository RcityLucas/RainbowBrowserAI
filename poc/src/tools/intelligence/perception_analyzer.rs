// Perception Analyzer Tool - Intelligent page analysis using perception engines
//
// This tool wraps the perception modules into the standard tools interface

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thirtyfour::WebDriver;
use tracing::{info, warn};

use crate::perception_mvp::{PerceptionEngineMVP, PageType as MVPPageType};
use crate::perception_simple::{SimplePerception, PageType as SimplePageType};
use crate::tools::{Tool, ToolError};

/// Perception analyzer tool that combines multiple perception engines
pub struct PerceptionAnalyzer {
    driver: Arc<WebDriver>,
    name: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionAnalyzerInput {
    /// URL to analyze (optional - uses current page if not provided)
    pub url: Option<String>,
    /// Which perception engine to use
    pub engine: PerceptionEngine,
    /// What to analyze
    pub analysis_type: AnalysisType,
    /// Element description for find operations
    pub element_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionEngine {
    MVP,        // Advanced MVP perception
    Simple,     // Simple heuristic perception
    Combined,   // Use both and merge results
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    ClassifyPage,      // Determine page type
    FindElement,       // Find specific element
    ExtractData,       // Extract structured data
    AnalyzeLayout,     // Analyze page layout
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionAnalyzerOutput {
    pub success: bool,
    pub engine_used: String,
    pub page_type: Option<String>,
    pub elements: Vec<PerceptionElement>,
    pub data: Option<serde_json::Value>,
    pub confidence: f32,
    pub execution_time_ms: u64,
    pub insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionElement {
    pub selector: String,
    pub text: String,
    pub element_type: String,
    pub confidence: f32,
    pub position: Option<ElementPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl PerceptionAnalyzer {
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self {
            driver,
            name: "perception_analyzer".to_string(),
            description: "Intelligent page analysis using multiple perception engines".to_string(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Classify page using specified engine
    async fn classify_page(&self, engine: &PerceptionEngine) -> Result<(String, String, f32)> {
        match engine {
            PerceptionEngine::MVP => {
                let mut perception = PerceptionEngineMVP::new(self.driver.as_ref().clone());
                match perception.classify_page().await {
                    Ok(page_type) => Ok((format!("{:?}", page_type), "MVP".to_string(), 0.85)),
                    Err(e) => Err(anyhow::anyhow!("MVP classification failed: {}", e)),
                }
            }
            PerceptionEngine::Simple => {
                let mut perception = SimplePerception::new(self.driver.as_ref().clone());
                match perception.classify_page().await {
                    Ok(page_type) => Ok((format!("{:?}", page_type), "Simple".to_string(), 0.75)),
                    Err(e) => Err(anyhow::anyhow!("Simple classification failed: {}", e)),
                }
            }
            PerceptionEngine::Combined => {
                // Try MVP first, fallback to Simple
                let mut mvp_perception = PerceptionEngineMVP::new(self.driver.as_ref().clone());
                match mvp_perception.classify_page().await {
                    Ok(page_type) => Ok((format!("{:?}", page_type), "MVP".to_string(), 0.85)),
                    Err(_) => {
                        info!("MVP failed, trying Simple perception");
                        let mut simple_perception = SimplePerception::new(self.driver.as_ref().clone());
                        match simple_perception.classify_page().await {
                            Ok(page_type) => Ok((format!("{:?}", page_type), "Simple".to_string(), 0.65)),
                            Err(e) => Err(anyhow::anyhow!("Both perception engines failed: {}", e)),
                        }
                    }
                }
            }
        }
    }

    /// Find element using specified engine
    async fn find_element(&self, engine: &PerceptionEngine, description: &str) -> Result<(Vec<PerceptionElement>, String, f32)> {
        match engine {
            PerceptionEngine::MVP => {
                let mut perception = PerceptionEngineMVP::new(self.driver.as_ref().clone());
                match perception.find_element(description).await {
                    Ok(element) => {
                        let perception_element = PerceptionElement {
                            selector: element.selector,
                            text: element.text,
                            element_type: format!("{:?}", element.element_type),
                            confidence: element.confidence,
                            position: None, // TODO: Add position detection
                        };
                        Ok((vec![perception_element], "MVP".to_string(), element.confidence))
                    }
                    Err(e) => Err(anyhow::anyhow!("MVP element search failed: {}", e)),
                }
            }
            PerceptionEngine::Simple => {
                let mut perception = SimplePerception::new(self.driver.as_ref().clone());
                match perception.find_element(description).await {
                    Ok(element) => {
                        let perception_element = PerceptionElement {
                            selector: element.selector,
                            text: element.text,
                            element_type: element.element_type,
                            confidence: element.confidence,
                            position: None,
                        };
                        Ok((vec![perception_element], "Simple".to_string(), element.confidence))
                    }
                    Err(e) => Err(anyhow::anyhow!("Simple element search failed: {}", e)),
                }
            }
            PerceptionEngine::Combined => {
                // Try MVP first, then Simple
                let mut mvp_perception = PerceptionEngineMVP::new(self.driver.as_ref().clone());
                match mvp_perception.find_element(description).await {
                    Ok(element) => {
                        let perception_element = PerceptionElement {
                            selector: element.selector,
                            text: element.text,
                            element_type: format!("{:?}", element.element_type),
                            confidence: element.confidence,
                            position: None,
                        };
                        Ok((vec![perception_element], "MVP".to_string(), element.confidence))
                    }
                    Err(_) => {
                        info!("MVP element search failed, trying Simple");
                        let mut simple_perception = SimplePerception::new(self.driver.as_ref().clone());
                        match simple_perception.find_element(description).await {
                            Ok(element) => {
                                let perception_element = PerceptionElement {
                                    selector: element.selector,
                                    text: element.text,
                                    element_type: element.element_type,
                                    confidence: element.confidence * 0.8, // Reduce confidence for fallback
                                    position: None,
                                };
                                Ok((vec![perception_element], "Simple (fallback)".to_string(), element.confidence * 0.8))
                            }
                            Err(e) => Err(anyhow::anyhow!("Both engines failed to find element: {}", e)),
                        }
                    }
                }
            }
        }
    }

    /// Extract data from current page
    async fn extract_data(&self, engine: &PerceptionEngine) -> Result<(serde_json::Value, String, f32)> {
        match engine {
            PerceptionEngine::MVP => {
                let mut perception = PerceptionEngineMVP::new(self.driver.as_ref().clone());
                
                // Get page type
                let page_type = perception.classify_page().await
                    .unwrap_or(MVPPageType::Unknown);
                
                // Extract common elements
                let mut data = serde_json::json!({
                    "page_type": format!("{:?}", page_type),
                    "extraction_method": "MVP",
                    "elements": []
                });

                // Try to extract common elements
                let common_elements = ["title", "heading", "link", "button", "form"];
                let mut found_elements = Vec::new();
                
                for element_type in &common_elements {
                    if let Ok(element) = perception.find_element(element_type).await {
                        found_elements.push(serde_json::json!({
                            "type": element_type,
                            "text": element.text,
                            "selector": element.selector,
                            "confidence": element.confidence
                        }));
                    }
                }

                data["elements"] = serde_json::Value::Array(found_elements);
                Ok((data, "MVP".to_string(), 0.8))
            }
            _ => {
                // For Simple and Combined, use basic extraction
                let data = serde_json::json!({
                    "page_type": "Unknown",
                    "extraction_method": "Basic",
                    "message": "Full data extraction only available with MVP engine"
                });
                Ok((data, "Basic".to_string(), 0.5))
            }
        }
    }
}

#[async_trait]
impl Tool for PerceptionAnalyzer {
    type Input = PerceptionAnalyzerInput;
    type Output = PerceptionAnalyzerOutput;

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let start_time = std::time::Instant::now();
        
        // Navigate to URL if provided
        if let Some(url) = &params.url {
            info!("Navigating to: {}", url);
            self.driver.get(url).await
                .map_err(|e| anyhow::anyhow!("Navigation failed: {}", e))?;
            
            // Wait for page to load
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        let mut output = PerceptionAnalyzerOutput {
            success: false,
            engine_used: "None".to_string(),
            page_type: None,
            elements: Vec::new(),
            data: None,
            confidence: 0.0,
            execution_time_ms: 0,
            insights: Vec::new(),
        };

        // Execute based on analysis type
        match params.analysis_type {
            AnalysisType::ClassifyPage => {
                match self.classify_page(&params.engine).await {
                    Ok((page_type, engine, confidence)) => {
                        output.success = true;
                        output.engine_used = engine;
                        output.page_type = Some(page_type);
                        output.confidence = confidence;
                        output.insights.push("Page classification completed successfully".to_string());
                    }
                    Err(e) => {
                        warn!("Page classification failed: {}", e);
                        output.insights.push(format!("Classification failed: {}", e));
                    }
                }
            }
            
            AnalysisType::FindElement => {
                if let Some(description) = params.element_description {
                    match self.find_element(&params.engine, &description).await {
                        Ok((elements, engine, confidence)) => {
                            output.success = true;
                            output.engine_used = engine;
                            output.elements = elements;
                            output.confidence = confidence;
                            output.insights.push(format!("Found {} elements matching '{}'", output.elements.len(), description));
                        }
                        Err(e) => {
                            warn!("Element search failed: {}", e);
                            output.insights.push(format!("Element search failed: {}", e));
                        }
                    }
                } else {
                    output.insights.push("Element description required for FindElement analysis".to_string());
                }
            }

            AnalysisType::ExtractData => {
                match self.extract_data(&params.engine).await {
                    Ok((data, engine, confidence)) => {
                        output.success = true;
                        output.engine_used = engine;
                        output.data = Some(data);
                        output.confidence = confidence;
                        output.insights.push("Data extraction completed".to_string());
                    }
                    Err(e) => {
                        warn!("Data extraction failed: {}", e);
                        output.insights.push(format!("Data extraction failed: {}", e));
                    }
                }
            }

            AnalysisType::AnalyzeLayout => {
                // TODO: Implement layout analysis
                output.insights.push("Layout analysis not yet implemented".to_string());
            }
        }

        output.execution_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(output)
    }

    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        match params.analysis_type {
            AnalysisType::FindElement if params.element_description.is_none() => {
                Err(anyhow::anyhow!("Element description required for FindElement analysis"))
            }
            _ => Ok(())
        }
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to analyze (optional)"
                },
                "engine": {
                    "type": "string",
                    "enum": ["MVP", "Simple", "Combined"],
                    "description": "Perception engine to use"
                },
                "analysis_type": {
                    "type": "string",
                    "enum": ["ClassifyPage", "FindElement", "ExtractData", "AnalyzeLayout"],
                    "description": "Type of analysis to perform"
                },
                "element_description": {
                    "type": "string",
                    "description": "Element description for find operations"
                }
            },
            "required": ["engine", "analysis_type"]
        })
    }

    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": {"type": "boolean"},
                "engine_used": {"type": "string"},
                "page_type": {"type": "string"},
                "elements": {"type": "array"},
                "data": {"type": "object"},
                "confidence": {"type": "number"},
                "execution_time_ms": {"type": "number"},
                "insights": {"type": "array"}
            }
        })
    }
}