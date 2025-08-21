// Interaction Tools Module
// Week 2 Implementation - Full TOOLS.md specification compliance

pub mod click;
pub mod type_text;
pub mod select_option;

#[cfg(test)]
mod tests;

// Re-export all interaction tools
pub use click::{Click, ClickParams, ClickResult};
pub use type_text::{TypeText, TypeTextParams, TypeTextResult};
pub use select_option::{SelectOption, SelectOptionParams, SelectOptionResult};

use super::{Tool, DynamicTool, DynamicToolWrapper};
use std::sync::Arc;

/// Create all interaction tools with the given browser instance
pub fn create_interaction_tools(browser: Arc<crate::browser::Browser>) -> Vec<Box<dyn DynamicTool>> {
    vec![
        Box::new(DynamicToolWrapper::new(Click::new(browser.clone()))),
        Box::new(DynamicToolWrapper::new(TypeText::new(browser.clone()))),
        Box::new(DynamicToolWrapper::new(SelectOption::new(browser.clone()))),
    ]
}