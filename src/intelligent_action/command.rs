// Command pattern for action execution - improves extensibility
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;

use super::{Action, ActionResult, executor::ActionExecutor};

/// Command trait for executable actions
#[async_trait]
pub trait ActionCommand: Send + Sync {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value>;
    async fn undo(&self, executor: &ActionExecutor) -> Result<()>;
    fn can_undo(&self) -> bool;
    fn name(&self) -> &str;
}

/// Click command implementation
pub struct ClickCommand {
    selector: String,
}

impl ClickCommand {
    pub fn new(selector: String) -> Self {
        Self { selector }
    }
}

#[async_trait]
impl ActionCommand for ClickCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Click { 
            selector: self.selector.clone() 
        }).await
    }

    async fn undo(&self, _executor: &ActionExecutor) -> Result<()> {
        // Click actions typically cannot be undone
        Err(anyhow::anyhow!("Click action cannot be undone"))
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Click"
    }
}

/// Input command implementation
pub struct InputCommand {
    selector: String,
    text: String,
    previous_value: Option<String>,
}

impl InputCommand {
    pub fn new(selector: String, text: String) -> Self {
        Self {
            selector,
            text,
            previous_value: None,
        }
    }

    pub fn with_previous_value(mut self, value: String) -> Self {
        self.previous_value = Some(value);
        self
    }
}

#[async_trait]
impl ActionCommand for InputCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Input {
            selector: self.selector.clone(),
            text: self.text.clone(),
        }).await
    }

    async fn undo(&self, executor: &ActionExecutor) -> Result<()> {
        if let Some(ref prev_value) = self.previous_value {
            executor.execute(&Action::Input {
                selector: self.selector.clone(),
                text: prev_value.clone(),
            }).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No previous value to restore"))
        }
    }

    fn can_undo(&self) -> bool {
        self.previous_value.is_some()
    }

    fn name(&self) -> &str {
        "Input"
    }
}

/// Navigate command implementation
pub struct NavigateCommand {
    url: String,
    previous_url: Option<String>,
}

impl NavigateCommand {
    pub fn new(url: String) -> Self {
        Self {
            url,
            previous_url: None,
        }
    }

    pub fn with_previous_url(mut self, url: String) -> Self {
        self.previous_url = Some(url);
        self
    }
}

#[async_trait]
impl ActionCommand for NavigateCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Navigate {
            url: self.url.clone(),
        }).await
    }

    async fn undo(&self, executor: &ActionExecutor) -> Result<()> {
        if let Some(ref prev_url) = self.previous_url {
            executor.execute(&Action::Navigate {
                url: prev_url.clone(),
            }).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No previous URL to navigate back to"))
        }
    }

    fn can_undo(&self) -> bool {
        self.previous_url.is_some()
    }

    fn name(&self) -> &str {
        "Navigate"
    }
}

/// Macro command for executing multiple commands
pub struct MacroCommand {
    commands: Vec<Arc<dyn ActionCommand>>,
    name: String,
}

impl MacroCommand {
    pub fn new(name: String) -> Self {
        Self {
            commands: Vec::new(),
            name,
        }
    }

    pub fn add_command(&mut self, command: Arc<dyn ActionCommand>) {
        self.commands.push(command);
    }
}

#[async_trait]
impl ActionCommand for MacroCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        let mut results = Vec::new();
        for command in &self.commands {
            let result = command.execute(executor).await?;
            results.push(result);
        }
        Ok(serde_json::json!(results))
    }

    async fn undo(&self, executor: &ActionExecutor) -> Result<()> {
        // Undo in reverse order
        for command in self.commands.iter().rev() {
            if command.can_undo() {
                command.undo(executor).await?;
            }
        }
        Ok(())
    }

    fn can_undo(&self) -> bool {
        self.commands.iter().all(|cmd| cmd.can_undo())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Command factory for creating commands from Action enum
pub struct CommandFactory {
    custom_commands: HashMap<String, Arc<dyn Fn(serde_json::Value) -> Arc<dyn ActionCommand> + Send + Sync>>,
}

impl CommandFactory {
    pub fn new() -> Self {
        Self {
            custom_commands: HashMap::new(),
        }
    }

    /// Create a command from an Action
    pub fn create_command(&self, action: &Action) -> Arc<dyn ActionCommand> {
        match action {
            Action::Click { selector } => {
                Arc::new(ClickCommand::new(selector.clone()))
            }
            Action::Input { selector, text } => {
                Arc::new(InputCommand::new(selector.clone(), text.clone()))
            }
            Action::Navigate { url } => {
                Arc::new(NavigateCommand::new(url.clone()))
            }
            Action::Wait { selector, timeout_ms, duration_ms } => {
                Arc::new(WaitCommand::new(selector.clone(), *timeout_ms, *duration_ms))
            }
            Action::Screenshot => {
                Arc::new(ScreenshotCommand::new())
            }
            Action::ExecuteScript { script } => {
                Arc::new(ScriptCommand::new(script.clone()))
            }
            Action::Scroll { direction, amount } => {
                Arc::new(ScrollCommand::new(*direction, *amount))
            }
            Action::Extract { selector } => {
                Arc::new(ExtractCommand::new(selector.clone()))
            }
            Action::Type { selector, text } => {
                Arc::new(TypeCommand::new(selector.clone(), text.clone()))
            }
        }
    }

    /// Register a custom command factory
    pub fn register_custom_command(
        &mut self,
        name: String,
        factory: Arc<dyn Fn(serde_json::Value) -> Arc<dyn ActionCommand> + Send + Sync>,
    ) {
        self.custom_commands.insert(name, factory);
    }

    /// Create a custom command by name
    pub fn create_custom_command(&self, name: &str, params: serde_json::Value) -> Option<Arc<dyn ActionCommand>> {
        self.custom_commands.get(name).map(|factory| factory(params))
    }
}

/// Additional command implementations
pub struct WaitCommand {
    selector: String,
    timeout_ms: u64,
    duration_ms: u64,
}

impl WaitCommand {
    pub fn new(selector: String, timeout_ms: u64, duration_ms: u64) -> Self {
        Self { selector, timeout_ms, duration_ms }
    }
}

#[async_trait]
impl ActionCommand for WaitCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Wait {
            selector: self.selector.clone(),
            timeout_ms: self.timeout_ms,
            duration_ms: self.duration_ms,
        }).await
    }

    async fn undo(&self, _executor: &ActionExecutor) -> Result<()> {
        Ok(()) // Wait has no undo
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Wait"
    }
}

pub struct ScreenshotCommand;

impl ScreenshotCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ActionCommand for ScreenshotCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Screenshot).await
    }

    async fn undo(&self, _executor: &ActionExecutor) -> Result<()> {
        Ok(()) // Screenshot has no undo
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Screenshot"
    }
}

pub struct ScriptCommand {
    script: String,
}

impl ScriptCommand {
    pub fn new(script: String) -> Self {
        Self { script }
    }
}

#[async_trait]
impl ActionCommand for ScriptCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::ExecuteScript {
            script: self.script.clone(),
        }).await
    }

    async fn undo(&self, _executor: &ActionExecutor) -> Result<()> {
        Err(anyhow::anyhow!("Script execution cannot be undone"))
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "ExecuteScript"
    }
}

pub struct ScrollCommand {
    direction: super::ScrollDirection,
    amount: i32,
}

impl ScrollCommand {
    pub fn new(direction: super::ScrollDirection, amount: i32) -> Self {
        Self { direction, amount }
    }
}

#[async_trait]
impl ActionCommand for ScrollCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Scroll {
            direction: self.direction,
            amount: self.amount,
        }).await
    }

    async fn undo(&self, executor: &ActionExecutor) -> Result<()> {
        // Undo by scrolling in opposite direction
        let opposite_direction = match self.direction {
            super::ScrollDirection::Up => super::ScrollDirection::Down,
            super::ScrollDirection::Down => super::ScrollDirection::Up,
            super::ScrollDirection::Left => super::ScrollDirection::Right,
            super::ScrollDirection::Right => super::ScrollDirection::Left,
        };
        
        executor.execute(&Action::Scroll {
            direction: opposite_direction,
            amount: self.amount,
        }).await?;
        Ok(())
    }

    fn can_undo(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "Scroll"
    }
}

pub struct ExtractCommand {
    selector: String,
}

impl ExtractCommand {
    pub fn new(selector: String) -> Self {
        Self { selector }
    }
}

#[async_trait]
impl ActionCommand for ExtractCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Extract {
            selector: self.selector.clone(),
        }).await
    }

    async fn undo(&self, _executor: &ActionExecutor) -> Result<()> {
        Ok(()) // Extract has no undo
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Extract"
    }
}

pub struct TypeCommand {
    selector: String,
    text: String,
}

impl TypeCommand {
    pub fn new(selector: String, text: String) -> Self {
        Self { selector, text }
    }
}

#[async_trait]
impl ActionCommand for TypeCommand {
    async fn execute(&self, executor: &ActionExecutor) -> Result<serde_json::Value> {
        executor.execute(&Action::Type {
            selector: self.selector.clone(),
            text: self.text.clone(),
        }).await
    }

    async fn undo(&self, _executor: &ActionExecutor) -> Result<()> {
        Err(anyhow::anyhow!("Type action cannot be undone"))
    }

    fn can_undo(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Type"
    }
}