//! Newtype ids shared across crates.

use serde::{Deserialize, Serialize};

/// Waydroid session serial / device id.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Serial(pub String);

/// Android package name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pkg(pub String);
