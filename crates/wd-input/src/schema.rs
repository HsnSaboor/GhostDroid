//! Canonical profile store.
//!
//! Port of phantom `profile.rs:13-22`.
#![deny(missing_docs)]

use crate::nodes::Node;

/// Screen override. Required non-zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Screen {
    /// Width px.
    pub width: u32,
    /// Height px.
    pub height: u32,
}

/// Canonical profile. One schema only.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    /// Name.
    pub name: String,
    /// Version (==1).
    pub version: u32,
    /// Screen.
    pub screen: Option<Screen>,
    /// Sensitivity > 0.
    #[serde(default = "crate::nodes::default_sens")]
    pub global_sensitivity: f64,
    /// Nodes.
    pub nodes: Vec<Node>,
}
