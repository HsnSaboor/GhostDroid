//! Screenshot pipeline (copy order, doc-first).
//!
//! 1. `adb-mcp/tools/media.py:_screencap`: `exec-out screencap -p`
//!    binary + CRLF→LF repair when first 64 B lack PNG magic.
//! 2. `scrcpy-mcp/tools/vision.ts`: scrcpy `frameBuffer` first,
//!    fallback adb raw→png; `structuredContent{source:scrcpy|adb}`.
//! 3. `us/tools/ui.ts:takeScreenshot`: png fast-path, else jpeg
//!    `quality`/`maxWidth` (no `sharp` dep here — weak i5).
//! 4. `waydroid-mcp/screen.py:strip_png` (cut warn text before
//!    `PNG_MAGIC`) + `raw_to_jpeg` trailing-junk tolerant.
//!
//! Returns MCP `image` base64 + `android://device/frame/latest.jpg`
//! at 2 FPS. Never fixed `compressed_screenshot.png` (multi-device race).

/// PNG magic bytes.
pub const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
/// Latest-frame resource URI.
pub const FRAME_RESOURCE: &str = "android://device/frame/latest.jpg";

/// `adb exec-out screencap -p` argv.
#[must_use]
pub fn screencap_args() -> Vec<String> {
    tracing::debug!("screenshot: args");
    vec!["exec-out".into(), "screencap".into(), "-p".into()]
}

/// Repair legacy LF→CRLF transports (check first 64 B, fix all).
#[must_use]
pub fn repair_crlf(data: Vec<u8>) -> Vec<u8> {
    tracing::debug!(bytes = data.len(), "screenshot: repair in");
    let head = &data[..data.len().min(64)];
    if !data.starts_with(PNG_MAGIC) && head.windows(2).any(|w| w == b"\r\n") {
        tracing::info!("screenshot: CRLF repaired");
        let out = strip_crlf(&data);
        tracing::debug!(bytes = out.len(), "screenshot: repair out");
        return out;
    }
    tracing::debug!(bytes = data.len(), "screenshot: repair out");
    data
}

/// Drop `\r` before every `\n`.
fn strip_crlf(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\r' && data.get(i + 1) == Some(&b'\n') {
            i += 1;
            continue;
        }
        out.push(data[i]);
        i += 1;
    }
    out
}

/// Cut warn text before `PNG_MAGIC` (`strip_png` port).
#[must_use]
pub fn strip_png_warn(data: &[u8]) -> &[u8] {
    tracing::debug!(bytes = data.len(), "screenshot: strip in");
    let mut i = 0;
    while i + PNG_MAGIC.len() <= data.len() {
        if &data[i..i + PNG_MAGIC.len()] == PNG_MAGIC {
            tracing::info!(offset = i, "screenshot: warn prefix cut");
            return &data[i..];
        }
        i += 1;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_and_strip() {
        let mut raw = b"WARN x\r\n".to_vec();
        raw.extend_from_slice(b"\x89PNG\r\n\x1a\nDATA");
        let stripped = strip_png_warn(&raw).to_vec();
        assert!(stripped.starts_with(PNG_MAGIC));
        let mut broken = Vec::with_capacity(stripped.len() * 2);
        for b in &stripped {
            if *b == b'\n' {
                broken.push(b'\r');
            }
            broken.push(*b);
        }
        assert!(repair_crlf(broken).starts_with(PNG_MAGIC));
    }
}
