use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Retry logic with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    operation: F,
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    let mut last_error = None;

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!("Attempt {}/{} failed: {}", attempt, max_attempts, e);
                last_error = Some(e);

                if attempt < max_attempts {
                    info!("Retrying in {:?}", delay);
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// Performance timer utility
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        if elapsed.as_millis() > 1000 {
            warn!("Slow operation '{}' took {}ms", self.operation, elapsed.as_millis());
        } else {
            info!("Operation '{}' completed in {}ms", self.operation, elapsed.as_millis());
        }
    }
}

/// URL validation and cleaning
pub fn clean_url(url: &str) -> String {
    let mut cleaned = url.trim().to_lowercase();
    
    // Add protocol if missing
    if !cleaned.starts_with("http://") && !cleaned.starts_with("https://") {
        cleaned = format!("https://{}", cleaned);
    }
    
    // Remove trailing slash
    if cleaned.ends_with('/') && cleaned.len() > 8 {
        cleaned.pop();
    }
    
    cleaned
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Generate safe filename from URL
pub fn url_to_filename(url: &str) -> String {
    let safe_chars: String = url
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    
    // Limit filename length
    if safe_chars.len() > 100 {
        format!("{}...", &safe_chars[..97])
    } else {
        safe_chars
    }
}

/// Cost calculation utilities
pub mod cost {
    /// Calculate OpenAI API cost based on tokens
    pub fn calculate_openai_cost(input_tokens: u32, output_tokens: u32, model: &str) -> f64 {
        let (input_rate, output_rate) = match model {
            "gpt-3.5-turbo" => (0.0005, 0.0015), // per 1K tokens
            "gpt-4" => (0.03, 0.06),
            "gpt-4-turbo" => (0.01, 0.03),
            _ => (0.001, 0.002), // default
        };
        
        (input_tokens as f64 / 1000.0) * input_rate + (output_tokens as f64 / 1000.0) * output_rate
    }
    
    /// Calculate Claude API cost based on tokens
    pub fn calculate_claude_cost(input_tokens: u32, output_tokens: u32, model: &str) -> f64 {
        let (input_rate, output_rate) = match model {
            "claude-3-haiku" => (0.00025, 0.00125),
            "claude-3-sonnet" => (0.003, 0.015),
            "claude-3-opus" => (0.015, 0.075),
            _ => (0.001, 0.005), // default
        };
        
        (input_tokens as f64 / 1000.0) * input_rate + (output_tokens as f64 / 1000.0) * output_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_url() {
        assert_eq!(clean_url("example.com"), "https://example.com");
        assert_eq!(clean_url("http://example.com/"), "http://example.com");
        assert_eq!(clean_url("HTTPS://EXAMPLE.COM"), "https://example.com");
    }

    #[test]
    fn test_url_to_filename() {
        assert_eq!(url_to_filename("https://example.com/path?query=1"), "https___example.com_path_query=1");
    }

    #[test]
    fn test_cost_calculation() {
        let cost = cost::calculate_openai_cost(1000, 500, "gpt-3.5-turbo");
        assert!((cost - 0.001250).abs() < 0.000001);
    }
}