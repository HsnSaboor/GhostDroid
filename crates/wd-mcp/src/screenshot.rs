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

/// Base64 alphabet value, or 255 when not in the alphabet.
const fn b64_val(byte: u8) -> u8 {
    match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => 255,
    }
}

/// Decode base64 (whitespace-tolerant, std-only, no new dep).
/// Returns raw bytes or an error string for callers to surface.
///
/// # Errors
///
/// Returns a message string when the input is empty, misaligned, or holds
/// non-alphabet bytes.
pub fn decode_b64(text: &str) -> Result<Vec<u8>, String> {
    let clean: Vec<u8> = text.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.is_empty() || !clean.len().is_multiple_of(4) {
        return Err("empty/invalid base64".to_owned());
    }
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    let mut i = 0;
    while i < clean.len() {
        let mut sextets = [0u8; 4];
        let mut pad = 0usize;
        for (j, chunk) in sextets.iter_mut().enumerate() {
            let byte = clean[i + j];
            if byte == b'=' {
                pad += 1;
            } else {
                let val = b64_val(byte);
                if val == 255 {
                    return Err("invalid base64 char".to_owned());
                }
                *chunk = val;
            }
        }
        let triple = (u32::from(sextets[0]) << 18)
            | (u32::from(sextets[1]) << 12)
            | (u32::from(sextets[2]) << 6)
            | u32::from(sextets[3]);
        // `triple` is a 24-bit base64 group; each shift+truncate extracts one
        // byte. Low 8 bits always hold the target byte, so no data is lost.
        out.push(u8::try_from((triple >> 16) & 0xFF).unwrap_or(0));
        if pad < 2 {
            out.push(u8::try_from((triple >> 8) & 0xFF).unwrap_or(0));
        }
        if pad < 1 {
            out.push(u8::try_from(triple & 0xFF).unwrap_or(0));
        }
        i += 4;
    }
    Ok(out)
}

/// Encode raw bytes as base64 (std-only, mirrors [`decode_b64`]).
#[must_use]
pub fn encode_b64(data: &[u8]) -> String {
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < data.len() {
        let a = data[i];
        let b = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let c = if i + 2 < data.len() { data[i + 2] } else { 0 };
        let triple = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(c);
        out.push(ALPHA[((triple >> 18) & 63) as usize] as char);
        out.push(ALPHA[((triple >> 12) & 63) as usize] as char);
        if i + 1 < data.len() {
            out.push(ALPHA[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < data.len() {
            out.push(ALPHA[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

/// Clean a `base64` screenshot: decode → cut warn prefix → CRLF
/// repair → require PNG magic → re-encode canonical (no whitespace).
/// Guarantees callers never forward warn text or CRLF-mangled PNGs.
///
/// # Errors
/// Returns a message string when the payload is not a decodable PNG.
pub fn clean_png_b64(b64: &str) -> Result<String, String> {
    tracing::debug!(len = b64.len(), "screenshot: clean in");
    let raw = decode_b64(b64)?;
    let stripped = strip_png_warn(&raw);
    let repaired = repair_crlf(stripped.to_vec());
    if !repaired.starts_with(PNG_MAGIC) {
        return Err("screenshot is not a PNG".to_owned());
    }
    let clean = encode_b64(&repaired);
    tracing::info!(bytes = clean.len(), "screenshot: clean out");
    Ok(clean)
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

    #[test]
    fn b64_roundtrip_and_clean() {
        let mut raw = b"WARN ".to_vec();
        raw.extend_from_slice(PNG_MAGIC);
        raw.extend_from_slice(b"DATA");
        let b64 = encode_b64(&raw);
        assert_eq!(decode_b64(&b64).unwrap(), raw);
        let clean = clean_png_b64(&b64).unwrap();
        let back = decode_b64(&clean).unwrap();
        assert!(back.starts_with(PNG_MAGIC));
        assert!(clean_png_b64("!!!").is_err());
        assert!(clean_png_b64(&encode_b64(b"nope")).is_err());
    }
}
