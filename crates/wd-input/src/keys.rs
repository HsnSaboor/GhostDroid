//! Key allowlist.
//!
//! Port of phantom `input.rs:Key::parse_name`.
#![deny(missing_docs)]

/// Known key? Accepts Qt `Key_X` / `LeftButton` import spellings.
#[must_use]
pub fn known(name: &str) -> bool {
    tracing::trace!(name, "wd-input: key check");
    canonical(name).is_some()
}

/// Normalize import spelling to canonical name.
#[must_use]
pub fn canonical(name: &str) -> Option<String> {
    let upper = name.trim().to_ascii_uppercase();
    if let Some(digit) = upper
        .strip_prefix("KEY")
        .filter(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_digit())
    {
        return Some(format!("Key{digit}"));
    }
    let bare = upper.strip_prefix("KEY_").unwrap_or(&upper);
    let mapped: &str = match bare {
        "CTRL" | "CONTROL" | "LEFTCTRL" => "LeftCtrl",
        "SHIFT" | "LEFTSHIFT" => "LeftShift",
        "ALT" | "LEFTALT" => "LeftAlt",
        "WIN" | "SUPER" | "LEFTMETA" => "LeftMeta",
        "ENTER" | "RETURN" => "Enter",
        "ESC" | "ESCAPE" => "Esc",
        "SPACE" | "SPACEBAR" => "Space",
        "TAB" => "Tab",
        "BACKSPACE" => "Backspace",
        "DELETE" => "Delete",
        "INSERT" => "Insert",
        "UP" => "Up",
        "DOWN" => "Down",
        "LEFT" => "Left",
        "RIGHT" => "Right",
        "HOME" => "Home",
        "END" => "End",
        "PAGEUP" => "PageUp",
        "PAGEDOWN" => "PageDown",
        "CAPSLOCK" | "CAPS_LOCK" => "CapsLock",
        "MINUS" => "Minus",
        "EQUAL" | "PLUS" => "Equal",
        "LEFTBRACE" => "LeftBrace",
        "RIGHTBRACE" => "RightBrace",
        "SEMICOLON" => "Semicolon",
        "APOSTROPHE" | "QUOTELEFT" | "QUOTE" => "Apostrophe",
        "GRAVE" => "Grave",
        "BACKSLASH" => "Backslash",
        "COMMA" => "Comma",
        "DOT" | "PERIOD" => "Dot",
        "SLASH" => "Slash",
        "LEFTBUTTON" | "MOUSELEFT" => "MouseLeft",
        "RIGHTBUTTON" | "MOUSERIGHT" => "MouseRight",
        "MIDDLEBUTTON" | "MOUSEMIDDLE" => "MouseMiddle",
        "WHEELUP" => "WheelUp",
        "WHEELDOWN" => "WheelDown",
        "RIGHTCTRL" => "RightCtrl",
        "RIGHTSHIFT" => "RightShift",
        "RIGHTALT" => "RightAlt",
        "RIGHTMETA" => "RightMeta",
        b if b.len() == 1 && b.as_bytes()[0].is_ascii_alphanumeric() => {
            return Some(canonical_alnum(b));
        }
        b if is_letter(b) => return Some(b.to_string()),
        b if is_fn_key(b) => return Some(b.to_string()),
        b if is_kp_key(b) => return Some(b.to_string()),
        _ => return None,
    };
    Some(mapped.to_string())
}

fn canonical_alnum(b: &str) -> String {
    if b == "0" {
        "Key0".to_string()
    } else if b.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("Key{b}")
    } else {
        b.to_string()
    }
}

fn is_letter(b: &str) -> bool {
    matches!(
        b,
        "A" | "B"
            | "C"
            | "D"
            | "E"
            | "F"
            | "G"
            | "H"
            | "I"
            | "J"
            | "K"
            | "L"
            | "M"
            | "N"
            | "O"
            | "P"
            | "Q"
            | "R"
            | "S"
            | "T"
            | "U"
            | "V"
            | "W"
            | "X"
            | "Y"
            | "Z"
    )
}

fn is_fn_key(b: &str) -> bool {
    matches!(
        b,
        "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11" | "F12"
    )
}

fn is_kp_key(b: &str) -> bool {
    matches!(
        b,
        "KP0"
            | "KP1"
            | "KP2"
            | "KP3"
            | "KP4"
            | "KP5"
            | "KP6"
            | "KP7"
            | "KP8"
            | "KP9"
            | "NUMLOCK"
            | "SCROLLLOCK"
            | "SYSRQ"
            | "PAUSE"
    )
}
