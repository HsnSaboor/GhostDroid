//! First-frame wait support: task visibility over `adb dumpsys`.
//!
//! Heavy games (UE4 splash) use a translucent theme and draw nothing until
//! the engine boots — minutes under Houdini translation. The host
//! multi-window surface stays transparent meanwhile, which reads as
//! "broken" when launch reports instant success. These pure builders plus
//! the [`parse_task_visibility`] parser let callers poll until the task
//! flips `visible=true` and report honest boot state instead.
//!
//! Refs: `dumpsys activity activities` `Task{...}` lines, e.g.
//! `* Task{aadb42a #14 ... A=10139:com.tencent.ig U=0 visible=true ... translucent=false sz=1}`.

/// Visibility snapshot for one package's task(s).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskVis {
    /// Any matching task reports `visible=true`.
    pub visible: bool,
    /// All matching tasks still report `translucent=true`.
    pub translucent: bool,
}

/// Build `adb shell dumpsys activity activities` args (no `adb` prefix).
#[must_use]
pub fn dumpsys_args() -> Vec<String> {
    tracing::debug!("focus: dumpsys args");
    vec![
        "shell".to_owned(),
        "dumpsys".to_owned(),
        "activity".to_owned(),
        "activities".to_owned(),
    ]
}

/// Build `adb shell pidof <pkg>` args (no `adb` prefix).
#[must_use]
pub fn pidof_args(pkg: &str) -> Vec<String> {
    tracing::debug!(pkg, "focus: pidof args");
    vec!["shell".to_owned(), "pidof".to_owned(), pkg.to_owned()]
}

/// Read one `key=true|false` flag from a `Task{...}` line.
fn flag(line: &str, key: &str) -> Option<bool> {
    let needle = format!("{key}=");
    let rest = line.split(needle.as_str()).nth(1)?;
    let token: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
    match token.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Parse task visibility for `pkg` from `dumpsys activity` output.
///
/// Matches `Task{...}` lines containing `:<pkg> ` (the `A=<uid>:<pkg>`
/// component). Folds multiple tasks: `visible` when any task is visible,
/// `translucent` only when every task is translucent. Returns [`None`]
/// when no task mentions the package.
#[must_use]
pub fn parse_task_visibility(dumpsys: &str, pkg: &str) -> Option<TaskVis> {
    tracing::debug!(pkg, "focus: parse task visibility");
    let needle = format!(":{pkg} ");
    let mut found = false;
    let mut visible = false;
    let mut translucent = true;
    for line in dumpsys.lines() {
        if !(line.contains("Task{") && line.contains(needle.as_str())) {
            continue;
        }
        found = true;
        if flag(line, "visible") == Some(true) {
            visible = true;
        }
        if flag(line, "translucent") == Some(false) {
            translucent = false;
        }
    }
    if found {
        Some(TaskVis {
            visible,
            translucent,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPLASH: &str = "* Task{aadb42a #14 type=standard A=10139:com.tencent.ig U=0 visible=false visibleRequested=false mode=fullscreen translucent=true sz=1}";
    const LOBBY: &str = "* Task{aadb42a #14 type=standard A=10139:com.tencent.ig U=0 visible=true visibleRequested=true mode=fullscreen translucent=false sz=1}";
    const OTHER: &str = "* Task{9c1 #7 type=standard A=10042:com.android.settings U=0 visible=true visibleRequested=true mode=fullscreen translucent=false sz=1}";

    #[test]
    fn splash_is_invisible_translucent() {
        let vis = parse_task_visibility(SPLASH, "com.tencent.ig").expect("task parses");
        assert!(!vis.visible);
        assert!(vis.translucent);
    }

    #[test]
    fn lobby_is_visible_opaque() {
        let vis = parse_task_visibility(LOBBY, "com.tencent.ig").expect("task parses");
        assert!(vis.visible);
        assert!(!vis.translucent);
    }

    #[test]
    fn unknown_pkg_is_none() {
        assert_eq!(parse_task_visibility(SPLASH, "com.example.game"), None);
    }

    #[test]
    fn other_tasks_do_not_leak() {
        let dump = format!("{SPLASH}\n{OTHER}");
        let vis = parse_task_visibility(&dump, "com.tencent.ig").expect("task parses");
        assert!(!vis.visible);
        assert!(vis.translucent);
    }

    #[test]
    fn any_visible_wins_fold() {
        let dump = format!("{SPLASH}\n{LOBBY}");
        let vis = parse_task_visibility(&dump, "com.tencent.ig").expect("task parses");
        assert!(vis.visible);
        assert!(!vis.translucent);
    }

    #[test]
    fn builders_shape() {
        assert_eq!(
            dumpsys_args(),
            vec!["shell", "dumpsys", "activity", "activities"]
        );
        assert_eq!(
            pidof_args("com.tencent.ig"),
            vec!["shell", "pidof", "com.tencent.ig"]
        );
    }
}
