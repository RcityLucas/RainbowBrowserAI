// Feature flag management - centralized feature configuration

/// Feature modules organization
pub struct Features;

impl Features {
    /// Check if standalone feature is enabled
    pub const fn is_standalone_enabled() -> bool {
        cfg!(feature = "standalone")
    }
}

// Conditional module imports based on features
#[cfg(feature = "standalone")]
pub mod standalone {
    // Re-export standalone browser functionality when feature is enabled
}

/// Feature-gated initialization
pub fn initialize_features() {
    #[cfg(feature = "standalone")]
    {
        log::info!("Standalone browser feature enabled");
    }
}