// API module - HTTP endpoints and request handling
pub mod api;
pub mod api_optimized;
pub mod api_optimized_simple;
pub mod api_perception;
pub mod api_v2;

// Re-export main types
pub use api::*;
pub use api_optimized::{OptimizedApiState, create_optimized_routes};
pub use api_optimized_simple::{SimplifiedApiState, create_simplified_routes};
pub use api_perception::*;