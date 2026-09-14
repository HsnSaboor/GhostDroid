//! JSON-RPC request/response envelope.

use serde::{Deserialize, Serialize};

/// Generic daemon request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Req {
    /// Method name.
    pub method: String,
    /// Params payload.
    pub params: serde_json::Value,
}

/// Generic daemon response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resp {
    /// Success flag.
    pub ok: bool,
    /// Data payload.
    pub data: serde_json::Value,
}

impl Resp {
    /// Success response.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn ok(data: serde_json::Value) -> Self {
        Self { ok: true, data }
    }

    /// Error response.
    #[must_use]
    pub fn err(msg: &str) -> Self {
        Self {
            ok: false,
            data: serde_json::Value::String(msg.to_owned()),
        }
    }
}
