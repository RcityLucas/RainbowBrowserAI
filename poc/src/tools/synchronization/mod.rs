// Synchronization tools for waiting and condition checking
// 
// This module contains tools that help coordinate timing and waiting
// in browser automation workflows.

pub mod wait_for_element;
pub mod wait_for_condition;

// Re-export tools for easy access
pub use wait_for_element::{WaitForElement, WaitForElementInput, WaitForElementOutput};
pub use wait_for_condition::{WaitForCondition, WaitForConditionInput, WaitForConditionOutput};

// Common types used by synchronization tools
use std::time::Duration;

/// Element states that can be waited for
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ElementState {
    /// Element exists in the DOM
    Attached,
    /// Element has been removed from the DOM
    Detached,
    /// Element is visible to the user
    Visible,
    /// Element is present but not visible
    Hidden,
    /// Element can be interacted with
    Enabled,
    /// Element exists but is disabled
    Disabled,
}

/// Wait strategies for different scenarios
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WaitStrategy {
    /// Maximum time to wait in milliseconds
    pub timeout_ms: u64,
    /// How often to check the condition in milliseconds
    pub poll_interval_ms: u64,
    /// Whether to throw error on timeout
    pub throw_on_timeout: bool,
}

impl Default for WaitStrategy {
    fn default() -> Self {
        Self {
            timeout_ms: 30000, // 30 seconds
            poll_interval_ms: 100, // 100 milliseconds
            throw_on_timeout: true,
        }
    }
}

/// Result of a wait operation
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WaitResult<T> {
    /// Whether the wait was successful
    pub success: bool,
    /// Value obtained (if any)
    pub value: Option<T>,
    /// Time spent waiting in milliseconds
    pub duration_ms: u64,
    /// Number of polling attempts made
    pub attempts: u32,
}