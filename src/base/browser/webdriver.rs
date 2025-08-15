//! # WebDriver实现 - 真实的浏览器控制
//! 
//! 使用 thirtyfour 库提供真实的浏览器自动化功能

use std::time::Duration;
use anyhow::Result;
use thirtyfour::{prelude::*, DesiredCapabilities};
use serde::{Deserialize, Serialize};

/// WebDriver浏览器控制器
pub struct WebDriverController {
    driver: Option<WebDriver>,
    config: WebDriverConfig,
    is_running: bool,
}

/// WebDriver配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDriverConfig {
    pub webdriver_url: String,
    pub browser: BrowserType,
    pub headless: bool,
    pub window_size: (u32, u32),
    pub user_agent: Option<String>,
    pub timeout: Duration,
}

/// 浏览器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Edge,
    Safari,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self {
            webdriver_url: "http://localhost:9515".to_string(), // ChromeDriver默认端口
            browser: BrowserType::Chrome,
            headless: false,
            window_size: (1920, 1080),
            user_agent: Some("RainbowBrowser/8.0 (Real WebDriver)".to_string()),
            timeout: Duration::from_secs(30),
        }
    }
}

impl WebDriverController {
    /// 创建新的WebDriver控制器
    pub async fn new(config: WebDriverConfig) -> Result<Self> {
        Ok(Self {
            driver: None,
            config,
            is_running: false,
        })
    }

    /// 启动浏览器
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Ok(());
        }

        log::info!("启动真实浏览器 (WebDriver)");
        
        // 创建浏览器capabilities
        let mut caps = match self.config.browser {
            BrowserType::Chrome => {
                let mut caps = DesiredCapabilities::chrome();
                if self.config.headless {
                    caps.add_chrome_arg("--headless")?;
                }
                caps.add_chrome_arg(&format!("--window-size={},{}", 
                    self.config.window_size.0, 
                    self.config.window_size.1))?;
                
                if let Some(ref ua) = self.config.user_agent {
                    caps.add_chrome_arg(&format!("--user-agent={}", ua))?;
                }
                
                // 禁用图片加载以提高速度
                caps.add_chrome_arg("--disable-images")?;
                caps.add_chrome_arg("--disable-gpu")?;
                caps.add_chrome_arg("--no-sandbox")?;
                caps.add_chrome_arg("--disable-dev-shm-usage")?;
                
                caps
            }
            BrowserType::Firefox => {
                let mut caps = DesiredCapabilities::firefox();
                if self.config.headless {
                    caps.add_firefox_arg("-headless")?;
                }
                caps
            }
            BrowserType::Edge => DesiredCapabilities::edge(),
            BrowserType::Safari => DesiredCapabilities::safari(),
        };

        // 连接到WebDriver服务器
        match WebDriver::new(&self.config.webdriver_url, caps).await {
            Ok(driver) => {
                self.driver = Some(driver);
                self.is_running = true;
                log::info!("✅ 成功连接到WebDriver");
                Ok(())
            }
            Err(e) => {
                log::error!("❌ 无法连接到WebDriver: {}", e);
                log::info!("请确保WebDriver服务正在运行:");
                log::info!("  Chrome: 运行 'chromedriver --port=9515'");
                log::info!("  Firefox: 运行 'geckodriver --port=4444'");
                Err(e.into())
            }
        }
    }

    /// 导航到URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        if let Some(ref driver) = self.driver {
            log::info!("🌐 导航到: {}", url);
            driver.goto(url).await?;
            // 等待页面加载
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ok(())
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 查找元素
    pub async fn find_element(&self, selector: &str) -> Result<WebElement> {
        if let Some(ref driver) = self.driver {
            // 尝试多种选择器类型
            if selector.starts_with('#') {
                // ID选择器
                let id = &selector[1..];
                Ok(driver.find(By::Id(id)).await?)
            } else if selector.starts_with('.') {
                // Class选择器
                let class = &selector[1..];
                Ok(driver.find(By::ClassName(class)).await?)
            } else if selector.contains('[') && selector.contains(']') {
                // CSS选择器
                Ok(driver.find(By::Css(selector)).await?)
            } else if selector.starts_with("//") {
                // XPath选择器
                Ok(driver.find(By::XPath(selector)).await?)
            } else {
                // 默认作为CSS选择器
                Ok(driver.find(By::Css(selector)).await?)
            }
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 点击元素
    pub async fn click(&self, selector: &str) -> Result<()> {
        log::info!("🖱️ 点击元素: {}", selector);
        let element = self.find_element(selector).await?;
        element.click().await?;
        Ok(())
    }

    /// 输入文本
    pub async fn input_text(&self, selector: &str, text: &str) -> Result<()> {
        log::info!("⌨️ 在 {} 输入: {}", selector, text);
        let element = self.find_element(selector).await?;
        element.clear().await?;
        element.send_keys(text).await?;
        Ok(())
    }

    /// 获取元素文本
    pub async fn get_text(&self, selector: &str) -> Result<String> {
        let element = self.find_element(selector).await?;
        Ok(element.text().await?)
    }

    /// 执行JavaScript
    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        if let Some(ref driver) = self.driver {
            let result = driver.execute(script, vec![]).await?;
            Ok(result)
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 截图
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        if let Some(ref driver) = self.driver {
            Ok(driver.screenshot_as_png().await?)
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 获取当前URL
    pub async fn current_url(&self) -> Result<String> {
        if let Some(ref driver) = self.driver {
            Ok(driver.current_url().await?.to_string())
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 获取页面标题
    pub async fn page_title(&self) -> Result<String> {
        if let Some(ref driver) = self.driver {
            Ok(driver.title().await?)
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 等待元素出现
    pub async fn wait_for_element(&self, selector: &str, timeout_secs: u64) -> Result<WebElement> {
        if let Some(ref driver) = self.driver {
            log::info!("⏳ 等待元素: {} (超时: {}秒)", selector, timeout_secs);
            
            let by = if selector.starts_with('#') {
                By::Id(&selector[1..])
            } else if selector.starts_with('.') {
                By::ClassName(&selector[1..])
            } else {
                By::Css(selector)
            };
            
            driver.query(by)
                .wait(Duration::from_secs(timeout_secs), Duration::from_millis(500))
                .first()
                .await
                .map_err(|e| anyhow::anyhow!("等待元素超时: {}", e))
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 搜索Google
    pub async fn google_search(&self, query: &str) -> Result<Vec<String>> {
        // 导航到Google
        self.navigate("https://www.google.com").await?;
        
        // 输入搜索内容
        self.input_text("input[name='q']", query).await?;
        
        // 提交搜索
        self.execute_script("document.querySelector('input[name=\"q\"]').form.submit()").await?;
        
        // 等待结果加载
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // 获取搜索结果
        if let Some(ref driver) = self.driver {
            let results = driver.find_all(By::Css("h3")).await?;
            let mut titles = Vec::new();
            
            for result in results.iter().take(5) {
                if let Ok(text) = result.text().await {
                    titles.push(text);
                }
            }
            
            Ok(titles)
        } else {
            Err(anyhow::anyhow!("浏览器未启动"))
        }
    }

    /// 关闭浏览器
    pub async fn quit(&mut self) -> Result<()> {
        if let Some(driver) = self.driver.take() {
            log::info!("👋 关闭浏览器");
            driver.quit().await?;
            self.is_running = false;
        }
        Ok(())
    }

    /// 检查浏览器是否运行中
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

/// 自动管理WebDriver生命周期
impl Drop for WebDriverController {
    fn drop(&mut self) {
        if self.is_running {
            // 使用阻塞方式关闭浏览器
            let _ = tokio::runtime::Handle::current().block_on(async {
                self.quit().await
            });
        }
    }
}