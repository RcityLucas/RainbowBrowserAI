use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thirtyfour::{WebDriver, ChromeCapabilities, DesiredCapabilities, ChromiumLikeCapabilities};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::traits::*;
use crate::utils::{retry_with_backoff, Timer};

/// Browser Service implementation using WebDriver
pub struct WebDriverBrowserService {
    sessions: Arc<RwLock<HashMap<SessionId, BrowserSession>>>,
    pool_config: BrowserPoolConfig,
}

struct BrowserSession {
    driver: WebDriver,
    created_at: chrono::DateTime<chrono::Utc>,
    last_used: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct BrowserPoolConfig {
    pub max_sessions: usize,
    pub session_timeout: Duration,
    pub connection_retries: u32,
    pub connection_timeout: Duration,
    pub webdriver_url: String,
}

impl Default for BrowserPoolConfig {
    fn default() -> Self {
        Self {
            max_sessions: 10,
            session_timeout: Duration::from_secs(300), // 5 minutes
            connection_retries: 3,
            connection_timeout: Duration::from_secs(30),
            webdriver_url: "http://localhost:9515".to_string(),
        }
    }
}

impl WebDriverBrowserService {
    pub fn new(config: BrowserPoolConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pool_config: config,
        }
    }

    pub fn new_default() -> Self {
        Self::new(BrowserPoolConfig::default())
    }

    async fn create_webdriver(&self) -> Result<WebDriver> {
        let _timer = Timer::new("create_webdriver");
        
        let _caps = ChromeCapabilities::new();
        let mut desired_caps = DesiredCapabilities::chrome();
        
        // Configure Chrome options for better automation
        let chrome_args = vec![
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--disable-gpu",
            "--disable-background-timer-throttling",
            "--disable-backgrounding-occluded-windows",
            "--disable-renderer-backgrounding",
            "--disable-features=TranslateUI",
            "--disable-web-security",
            "--allow-running-insecure-content",
        ];
        
        for arg in chrome_args {
            desired_caps.add_arg(arg)?;
        }

        let webdriver_url = &self.pool_config.webdriver_url;
        
        retry_with_backoff(
            || {
                let caps = desired_caps.clone();
                let url = webdriver_url.clone();
                Box::pin(async move {
                    WebDriver::new(&url, caps).await
                })
            },
            self.pool_config.connection_retries,
            Duration::from_millis(500),
            Duration::from_secs(5),
        ).await.map_err(|e| anyhow::anyhow!("Failed to create WebDriver: {}", e))
    }

    async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now();
        let timeout = self.pool_config.session_timeout;
        
        let expired_sessions: Vec<SessionId> = sessions
            .iter()
            .filter(|(_, session)| {
                now.signed_duration_since(session.last_used).to_std().unwrap_or(Duration::ZERO) > timeout
            })
            .map(|(id, _)| *id)
            .collect();

        for session_id in expired_sessions {
            if let Some(session) = sessions.remove(&session_id) {
                warn!("Cleaning up expired session: {}", session_id);
                if let Err(e) = session.driver.quit().await {
                    error!("Failed to quit expired session driver: {}", e);
                }
            }
        }
    }

    async fn update_session_timestamp(&self, session_id: &SessionId) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_used = chrono::Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }
}

#[async_trait]
impl BrowserService for WebDriverBrowserService {
    async fn create_session(&self) -> Result<SessionId> {
        let _timer = Timer::new("create_session");
        
        // Clean up expired sessions first
        self.cleanup_expired_sessions().await;
        
        // Check if we've reached the maximum number of sessions
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.pool_config.max_sessions {
                return Err(anyhow::anyhow!("Maximum number of sessions reached"));
            }
        }

        // Create new WebDriver instance
        let driver = self.create_webdriver().await?;
        
        // Set timeouts
        driver.set_page_load_timeout(Duration::from_secs(30)).await?;
        driver.set_implicit_wait_timeout(Duration::from_secs(10)).await?;
        
        let session_id = Uuid::new_v4();
        let session = BrowserSession {
            driver,
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        info!("Created new browser session: {}", session_id);
        Ok(session_id)
    }

    async fn navigate(&self, session_id: &SessionId, url: &str) -> Result<NavigationResult> {
        let _timer = Timer::new("navigate");
        
        let cleaned_url = crate::utils::clean_url(url);
        if !crate::utils::is_valid_url(&cleaned_url) {
            return Err(anyhow::anyhow!("Invalid URL: {}", url));
        }

        // Update session timestamp
        self.update_session_timestamp(session_id).await?;
        
        let start = std::time::Instant::now();
        
        // Store original URL for fallback
        let fallback_url = cleaned_url.clone();
        
        // Navigate to URL with retry logic - Clone the sessions Arc to avoid lifetime issues
        let sessions_clone = self.sessions.clone();
        let session_id_clone = *session_id;
        
        retry_with_backoff(
            move || {
                let sessions = sessions_clone.clone();
                let url = cleaned_url.clone();
                let session_id = session_id_clone;
                Box::pin(async move {
                    let sessions_read = sessions.read().await;
                    if let Some(session) = sessions_read.get(&session_id) {
                        session.driver.goto(&url).await
                    } else {
                        Err(thirtyfour::error::WebDriverError::FatalError("Session not found".to_string()))
                    }
                })
            },
            3,
            Duration::from_millis(1000),
            Duration::from_secs(5),
        ).await?;

        let load_time_ms = start.elapsed().as_millis() as u64;

        // Get session again for reading page data
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Get page title
        let title = session.driver.title().await.ok();
        
        // Get current URL (might have redirected)
        let final_url = session.driver.current_url().await
            .map(|url| url.to_string())
            .unwrap_or_else(|_| fallback_url.clone());

        info!("Successfully navigated to: {} ({}ms)", final_url, load_time_ms);

        Ok(NavigationResult {
            url: final_url,
            title,
            status_code: None, // WebDriver doesn't provide HTTP status codes
            load_time_ms,
        })
    }

    async fn screenshot(&self, session_id: &SessionId, options: ScreenshotOptions) -> Result<Vec<u8>> {
        let _timer = Timer::new("screenshot");
        
        // Update session timestamp
        self.update_session_timestamp(session_id).await?;
        
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Set viewport size if specified
        if let (Some(width), Some(height)) = (options.viewport_width, options.viewport_height) {
            session.driver.set_window_rect(0, 0, width, height).await?;
            info!("Set viewport to {}x{}", width, height);
        }

        // Wait after load if specified
        if let Some(wait_time) = options.wait_after_load {
            tokio::time::sleep(wait_time).await;
        }

        // Take screenshot
        let screenshot_data = if options.full_page {
            // For full page screenshots, we need to get the full page height
            let body_height: u64 = session.driver
                .execute("return Math.max(document.body.scrollHeight, document.body.offsetHeight, document.documentElement.clientHeight, document.documentElement.scrollHeight, document.documentElement.offsetHeight);", vec![])
                .await?
                .json()
                .as_u64()
                .unwrap_or(1080);

            if let Some(width) = options.viewport_width {
                session.driver.set_window_rect(0, 0, width, body_height as u32).await?;
            }
            
            session.driver.screenshot_as_png().await?
        } else if let Some(selector) = &options.element_selector {
            // Screenshot of specific element
            let element = session.driver.find(thirtyfour::By::Css(selector)).await?;
            element.screenshot_as_png().await?
        } else {
            // Viewport screenshot
            session.driver.screenshot_as_png().await?
        };

        info!("Screenshot taken ({} bytes)", screenshot_data.len());
        Ok(screenshot_data)
    }

    async fn extract_content(&self, session_id: &SessionId) -> Result<PageContent> {
        let _timer = Timer::new("extract_content");
        
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Get basic page information
        let url = session.driver.current_url().await?.to_string();
        let title = session.driver.title().await.ok();

        // Extract meta description
        let meta_description = session.driver
            .find(thirtyfour::By::Css("meta[name='description']"))
            .await
            .ok()
            .and_then(|elem| {
                futures::executor::block_on(async {
                    elem.attr("content").await.ok().flatten()
                })
            });

        // Extract headings
        let headings = extract_text_from_elements(&session.driver, "h1, h2, h3, h4, h5, h6").await;

        // Extract text content from paragraphs
        let text_content = extract_text_from_elements(&session.driver, "p").await;

        // Extract links
        let links = extract_links(&session.driver).await;

        // Extract images
        let images = extract_images(&session.driver).await;

        // Extract forms
        let forms = extract_forms(&session.driver).await;

        Ok(PageContent {
            url,
            title,
            meta_description,
            headings,
            text_content,
            links,
            images,
            forms,
        })
    }

    async fn interact(&self, session_id: &SessionId, action: InteractionAction) -> Result<()> {
        let _timer = Timer::new("interact");
        
        // Update session timestamp
        self.update_session_timestamp(session_id).await?;
        
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        match action {
            InteractionAction::Click { selector } => {
                let element = session.driver.find(thirtyfour::By::Css(&selector)).await?;
                element.click().await?;
                info!("Clicked element: {}", selector);
            }
            InteractionAction::Type { selector, text } => {
                let element = session.driver.find(thirtyfour::By::Css(&selector)).await?;
                element.clear().await?;
                element.send_keys(&text).await?;
                info!("Typed text into element: {}", selector);
            }
            InteractionAction::Clear { selector } => {
                let element = session.driver.find(thirtyfour::By::Css(&selector)).await?;
                element.clear().await?;
                info!("Cleared element: {}", selector);
            }
            InteractionAction::Submit { selector } => {
                let element = session.driver.find(thirtyfour::By::Css(&selector)).await?;
                // Use JavaScript to submit the form as WebDriver submit() was removed
                session.driver.execute("arguments[0].form.submit();", vec![element.to_json()?]).await?;
                info!("Submitted form: {}", selector);
            }
            InteractionAction::Scroll { direction, amount } => {
                match direction {
                    ScrollDirection::Up => {
                        let amount = amount.unwrap_or(300);
                        session.driver.execute(&format!("window.scrollBy(0, {})", -amount), vec![]).await?;
                    }
                    ScrollDirection::Down => {
                        let amount = amount.unwrap_or(300);
                        session.driver.execute(&format!("window.scrollBy(0, {})", amount), vec![]).await?;
                    }
                    ScrollDirection::Left => {
                        let amount = amount.unwrap_or(300);
                        session.driver.execute(&format!("window.scrollBy({}, 0)", -amount), vec![]).await?;
                    }
                    ScrollDirection::Right => {
                        let amount = amount.unwrap_or(300);
                        session.driver.execute(&format!("window.scrollBy({}, 0)", amount), vec![]).await?;
                    }
                    ScrollDirection::ToElement { ref selector } => {
                        let element = session.driver.find(thirtyfour::By::Css(selector)).await?;
                        session.driver.execute("arguments[0].scrollIntoView();", vec![element.to_json()?]).await?;
                        info!("Scrolled to element: {}", selector);
                    }
                }
                info!("Scrolled: {:?}", direction);
            }
            InteractionAction::Wait { duration_ms } => {
                tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
                info!("Waited {}ms", duration_ms);
            }
            InteractionAction::WaitForElement { selector, timeout_ms } => {
                let timeout = std::time::Duration::from_millis(timeout_ms);
                session.driver.set_implicit_wait_timeout(timeout).await?;
                
                let start = std::time::Instant::now();
                while start.elapsed() < timeout {
                    if session.driver.find(thirtyfour::By::Css(&selector)).await.is_ok() {
                        info!("Element found: {}", selector);
                        return Ok(());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                
                return Err(anyhow::anyhow!("Element not found within timeout: {}", selector));
            }
        }

        Ok(())
    }

    async fn close_session(&self, session_id: &SessionId) -> Result<()> {
        let _timer = Timer::new("close_session");
        
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            session.driver.quit().await?;
            info!("Closed browser session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    async fn get_current_url(&self, session_id: &SessionId) -> Result<String> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        Ok(session.driver.current_url().await?.to_string())
    }

    async fn get_title(&self, session_id: &SessionId) -> Result<String> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        Ok(session.driver.title().await?)
    }
}

// Helper functions for content extraction
async fn extract_text_from_elements(driver: &WebDriver, selector: &str) -> Vec<String> {
    match driver.find_all(thirtyfour::By::Css(selector)).await {
        Ok(elements) => {
            let mut texts = Vec::new();
            for element in elements {
                if let Ok(text) = element.text().await {
                    if !text.trim().is_empty() {
                        texts.push(text.trim().to_string());
                    }
                }
            }
            texts
        }
        Err(_) => Vec::new(),
    }
}

async fn extract_links(driver: &WebDriver) -> Vec<LinkInfo> {
    match driver.find_all(thirtyfour::By::Css("a[href]")).await {
        Ok(elements) => {
            let mut links = Vec::new();
            for element in elements {
                if let (Ok(href), Ok(text)) = (element.attr("href").await, element.text().await) {
                    if let Some(href) = href {
                        let title = element.attr("title").await.ok().flatten();
                        links.push(LinkInfo {
                            text: text.trim().to_string(),
                            href,
                            title,
                        });
                    }
                }
            }
            links
        }
        Err(_) => Vec::new(),
    }
}

async fn extract_images(driver: &WebDriver) -> Vec<ImageInfo> {
    match driver.find_all(thirtyfour::By::Css("img[src]")).await {
        Ok(elements) => {
            let mut images = Vec::new();
            for element in elements {
                if let Ok(Some(src)) = element.attr("src").await {
                    let alt = element.attr("alt").await.ok().flatten();
                    let title = element.attr("title").await.ok().flatten();
                    images.push(ImageInfo { src, alt, title });
                }
            }
            images
        }
        Err(_) => Vec::new(),
    }
}

async fn extract_forms(driver: &WebDriver) -> Vec<FormInfo> {
    match driver.find_all(thirtyfour::By::Css("form")).await {
        Ok(elements) => {
            let mut forms = Vec::new();
            for element in elements {
                let action = element.attr("action").await.ok().flatten();
                let method = element.attr("method").await.ok()
                    .flatten()
                    .unwrap_or_else(|| "GET".to_string())
                    .to_uppercase();

                // Extract form fields
                let fields = if let Ok(field_elements) = element.find_all(thirtyfour::By::Css("input, select, textarea")).await {
                    let mut form_fields = Vec::new();
                    for field in field_elements {
                        let name = field.attr("name").await.ok().flatten();
                        let field_type = field.attr("type").await.ok()
                            .flatten()
                            .unwrap_or_else(|| "text".to_string());
                        let placeholder = field.attr("placeholder").await.ok().flatten();
                        let required = field.attr("required").await.ok().flatten().is_some();

                        form_fields.push(FormField {
                            name,
                            field_type,
                            placeholder,
                            required,
                        });
                    }
                    form_fields
                } else {
                    Vec::new()
                };

                forms.push(FormInfo {
                    action,
                    method,
                    fields,
                });
            }
            forms
        }
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_service_creation() {
        let service = WebDriverBrowserService::new_default();
        assert_eq!(service.pool_config.max_sessions, 10);
    }

    #[test]
    fn test_browser_pool_config() {
        let config = BrowserPoolConfig::default();
        assert_eq!(config.max_sessions, 10);
        assert_eq!(config.webdriver_url, "http://localhost:9515");
    }
}