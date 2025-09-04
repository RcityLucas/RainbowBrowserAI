use anyhow::{Result, anyhow};
use chromiumoxide::BrowserConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::info;
use super::core::{Browser, BrowserOps};

/// Browser session for stateful operations
#[derive(Clone)]
pub struct BrowserSession {
    pub id: String,
    pub browser: Arc<Browser>,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub current_url: Option<String>,
    pub history: Vec<String>,
}

impl BrowserSession {
    /// Create a new browser session
    pub async fn new() -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let browser = Arc::new(Browser::new().await?);
        
        info!("Created new browser session: {}", id);
        
        Ok(Self {
            id: id.clone(),
            browser,
            created_at: Utc::now(),
            last_used: Utc::now(),
            metadata: HashMap::new(),
            current_url: None,
            history: Vec::new(),
        })
    }

    /// Create session with custom browser config
    pub async fn with_config(config: BrowserConfig) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let browser = Arc::new(Browser::new_with_config(config).await?);
        
        info!("Created new browser session with custom config: {}", id);
        
        Ok(Self {
            id: id.clone(),
            browser,
            created_at: Utc::now(),
            last_used: Utc::now(),
            metadata: HashMap::new(),
            current_url: None,
            history: Vec::new(),
        })
    }

    /// Update last used timestamp
    pub fn touch(&mut self) {
        self.last_used = Utc::now();
    }

    /// Navigate and track history
    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        self.touch();
        self.browser.navigate_to(url).await?;
        
        // Update history
        if let Some(current) = &self.current_url {
            self.history.push(current.clone());
        }
        self.current_url = Some(url.to_string());
        
        info!("Session {} navigated to: {}", self.id, url);
        Ok(())
    }

    /// Go back in history
    pub async fn go_back(&mut self) -> Result<()> {
        self.touch();
        
        if let Some(previous_url) = self.history.pop() {
            let current = self.current_url.clone();
            self.browser.navigate_to(&previous_url).await?;
            self.current_url = Some(previous_url);
            
            // Don't add to history when going back
            if let Some(current) = current {
                info!("Session {} went back from {} to {}", 
                      self.id, current, self.current_url.as_ref().unwrap());
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("No history to go back to"))
        }
    }

    /// Get session age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get idle time in seconds
    pub fn idle_seconds(&self) -> i64 {
        (Utc::now() - self.last_used).num_seconds()
    }

    /// Check if session is expired (default 30 minutes idle)
    pub fn is_expired(&self, max_idle_seconds: i64) -> bool {
        self.idle_seconds() > max_idle_seconds
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Session manager for managing multiple browser sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<RwLock<BrowserSession>>>>>,
    max_sessions: usize,
    session_timeout: i64, // seconds
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_sessions: usize, session_timeout: i64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions,
            session_timeout,
        }
    }

    /// Create a new session
    pub async fn create_session(&self) -> Result<String> {
        // Clean up expired sessions first
        self.cleanup_expired().await;
        
        // Check if we've reached max sessions
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.max_sessions {
                return Err(anyhow::anyhow!(
                    "Maximum number of sessions ({}) reached", 
                    self.max_sessions
                ));
            }
        }
        
        // Create new session
        let session = BrowserSession::new().await?;
        let session_id = session.id.clone();
        
        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), Arc::new(RwLock::new(session)));
        
        info!("Created session: {} (total: {})", session_id, sessions.len());
        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<RwLock<BrowserSession>>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(session_id).is_some() {
            info!("Removed session: {} (remaining: {})", session_id, sessions.len());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();
        
        let expired_ids: Vec<String> = {
            let mut expired = Vec::new();
            for (id, session) in sessions.iter() {
                let session_guard = session.read().await;
                if session_guard.is_expired(self.session_timeout) {
                    expired.push(id.clone());
                }
            }
            expired
        };
        
        for id in &expired_ids {
            sessions.remove(id);
            info!("Cleaned up expired session: {}", id);
        }
        
        let removed_count = expired_ids.len();
        if removed_count > 0 {
            info!("Cleaned up {} expired sessions (remaining: {})", 
                  removed_count, sessions.len());
        }
        
        removed_count
    }

    /// Get all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let mut session_list = Vec::new();
        
        for (id, session) in sessions.iter() {
            let session_guard = session.read().await;
            session_list.push(SessionInfo {
                id: id.clone(),
                created_at: session_guard.created_at,
                last_used: session_guard.last_used,
                current_url: session_guard.current_url.clone(),
                age_seconds: session_guard.age_seconds(),
                idle_seconds: session_guard.idle_seconds(),
            });
        }
        
        session_list
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Clear all sessions
    pub async fn clear_all(&self) {
        let mut sessions = self.sessions.write().await;
        let count = sessions.len();
        sessions.clear();
        info!("Cleared all {} sessions", count);
    }
}

/// Session information for API responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub current_url: Option<String>,
    pub age_seconds: i64,
    pub idle_seconds: i64,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(10, 1800) // 10 sessions max, 30 minutes timeout
    }
}