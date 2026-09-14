//! Serial resolve: which device when many / none named.
//!
//! Port of `scrcpy-mcp/src/utils/adb.ts:resolveSerial/parseDeviceList`
//! and `Android-MCP/__main__.py:14-37` flags: explicit serial wins,
//! else `ANDROID_SERIAL` env, else single attached device, else
//! multi-device error. CRLF tolerated. No spawn here — pure parse.

/// One attached device line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceLine {
    /// Serial.
    pub serial: String,
    /// adb state (`device`, `offline`, ...).
    pub state: String,
}

/// Parse `adb devices -l` output. Tolerates CRLF.
#[must_use]
pub fn parse_device_list(out: &str) -> Vec<DeviceLine> {
    tracing::debug!(bytes = out.len(), "serial: parse in");
    let norm = out.replace("\r\n", "\n");
    let mut rows = Vec::new();
    for line in norm.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('*') {
            continue;
        }
        let mut parts = line.split_whitespace();
        if let (Some(serial), Some(state)) = (parts.next(), parts.next()) {
            rows.push(DeviceLine {
                serial: serial.to_owned(),
                state: state.to_owned(),
            });
        }
    }
    tracing::info!(count = rows.len(), "serial: parsed");
    rows
}

/// Pick a serial: explicit > `ANDROID_SERIAL` > single device.
/// Errors on multi-device ambiguity (caller maps to exit 4).
///
/// # Errors
/// Returns [`wd_core::WdError::Validation`] when ambiguous/empty.
pub fn resolve(explicit: Option<&str>, devices_out: &str) -> Result<String, wd_core::WdError> {
    tracing::info!(?explicit, "serial: resolve");
    if let Some(s) = explicit.filter(|s| !s.is_empty()) {
        tracing::debug!(serial = s, "serial: explicit wins");
        return Ok(s.to_owned());
    }
    if let Ok(env) = std::env::var("ANDROID_SERIAL")
        && !env.trim().is_empty()
    {
        tracing::debug!(serial = %env, "serial: env wins");
        return Ok(env);
    }
    let devices = parse_device_list(devices_out);
    let ready: Vec<&str> = devices
        .iter()
        .filter(|d| d.state == "device")
        .map(|d| d.serial.as_str())
        .collect();
    match ready.as_slice() {
        [one] => {
            tracing::info!(serial = one, "serial: single auto-pick");
            Ok((*one).to_owned())
        }
        [] => {
            tracing::warn!("serial: no devices");
            Err(wd_core::WdError::Validation("no devices attached".into()))
        }
        _ => {
            tracing::warn!(?ready, "serial: multi-device, need -s");
            Err(wd_core::WdError::Validation(
                "multiple devices: pass --device/-s or ANDROID_SERIAL".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_and_multi() {
        let out = "List of devices attached\r\nem-1\tdevice\r\nem-2\tdevice\r\n";
        assert_eq!(parse_device_list(out).len(), 2);
        assert!(resolve(None, out).is_err());
        assert_eq!(resolve(Some("em-1"), out).unwrap(), "em-1");
    }

    #[test]
    fn single_auto() {
        let out = "List of devices attached\nem-1\tdevice\n";
        assert_eq!(resolve(None, out).unwrap(), "em-1");
    }
}
