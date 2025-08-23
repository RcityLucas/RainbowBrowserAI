//! Navigation tools for browser control

pub mod navigate_to_url;
pub mod scroll_page;

pub use navigate_to_url::{NavigateToUrl, NavigateToUrlParams, NavigationOptions, NavigationResult};
pub use scroll_page::{ScrollPage, ScrollPageParams, ScrollOptions, ScrollResult};

#[cfg(test)]
mod tests;