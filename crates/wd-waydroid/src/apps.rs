//! App install/remove/launch/list arg builders.
//!
//! Refs: `.devdocs/waydroid/tools/__init__.py:89`,
//! `.devdocs/waydroid/tools/actions/app_manager.py:14-115`,
//! `.devdocs/waydroid/tools/helpers/arguments.py:76-89`.

use wd_core::Pkg;

/// Build `waydroid app install <apk-path>` args.
#[must_use]
pub fn install_args(apk_path: &str) -> Vec<String> {
    tracing::info!(apk_path, "apps: install args");
    vec!["app".to_owned(), "install".to_owned(), apk_path.to_owned()]
}

/// Build `waydroid app remove <package>` args.
#[must_use]
pub fn remove_args(pkg: &Pkg) -> Vec<String> {
    tracing::info!(package = %pkg.0, "apps: remove args");
    vec!["app".to_owned(), "remove".to_owned(), pkg.0.clone()]
}

/// Build `waydroid app launch <package>` args.
#[must_use]
pub fn launch_args(pkg: &Pkg) -> Vec<String> {
    tracing::info!(package = %pkg.0, "apps: launch args");
    vec!["app".to_owned(), "launch".to_owned(), pkg.0.clone()]
}

/// Launcher intent action (tap-equivalent resume path).
pub const LAUNCHER_ACTION: &str = "android.intent.action.MAIN";
/// Launcher intent category (tap-equivalent resume path).
pub const LAUNCHER_CATEGORY: &str = "android.intent.category.LAUNCHER";

/// Build `adb shell cmd package resolve-activity --brief` args that print
/// the package's launcher component on the last output line.
///
/// `waydroid app launch` alone can leave translucent-splash tasks stuck
/// invisible; re-firing the launcher intent (what a launcher tap sends)
/// resumes them. Pure builder; run via `adb`.
#[must_use]
pub fn resolve_launcher_args(pkg: &Pkg) -> Vec<String> {
    tracing::info!(package = %pkg.0, "apps: resolve launcher args");
    vec![
        "shell".to_owned(),
        "cmd".to_owned(),
        "package".to_owned(),
        "resolve-activity".to_owned(),
        "--brief".to_owned(),
        "-a".to_owned(),
        LAUNCHER_ACTION.to_owned(),
        "-c".to_owned(),
        LAUNCHER_CATEGORY.to_owned(),
        pkg.0.clone(),
    ]
}

/// Parse the launcher component (`pkg/Class`) from
/// [`resolve_launcher_args`] output: last non-empty line containing `/`.
/// Returns [`None`] when no launcher activity resolves. Pure, no spawn.
#[must_use]
pub fn parse_launcher_component(out: &str) -> Option<String> {
    tracing::debug!(len = out.len(), "apps: parse launcher component");
    out.lines()
        .map(str::trim)
        .rev()
        .find(|line| line.contains('/'))
        .map(ToOwned::to_owned)
}

/// Build `adb shell am start` args for a launcher component: the exact
/// tap-equivalent intent (`MAIN` + `LAUNCHER` + `-n`). Pure builder.
#[must_use]
pub fn launcher_start_args(component: &str) -> Vec<String> {
    tracing::info!(component, "apps: launcher start args");
    vec![
        "shell".to_owned(),
        "am".to_owned(),
        "start".to_owned(),
        "-a".to_owned(),
        LAUNCHER_ACTION.to_owned(),
        "-c".to_owned(),
        LAUNCHER_CATEGORY.to_owned(),
        "-n".to_owned(),
        component.to_owned(),
    ]
}

/// Build `waydroid app list` args.
#[must_use]
pub fn list_args() -> Vec<String> {
    tracing::info!("apps: list args");
    vec!["app".to_owned(), "list".to_owned()]
}

/// One row of `waydroid app list`: display name + package id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRow {
    /// Display name (`Name:` line).
    pub title: String,
    /// Package id (`packageName:` line).
    pub pkg: String,
}

/// Parse `waydroid app list` output into [`AppRow`]s.
///
/// Blocks look like `Name: X\npackageName: Y\ncategories:\n\t...`.
/// Skips blocks missing either line. Pure, no spawn.
#[must_use]
pub fn parse_app_list(out: &str) -> Vec<AppRow> {
    tracing::debug!(len = out.len(), "apps: parse list");
    let mut rows = Vec::new();
    let mut title: Option<String> = None;
    for line in out.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("Name:") {
            title = Some(name.trim().to_owned());
        } else if let Some(pkg) = trimmed.strip_prefix("packageName:")
            && let Some(t) = title.take()
        {
            let pkg = pkg.trim().to_owned();
            if !t.is_empty() && !pkg.is_empty() {
                rows.push(AppRow { title: t, pkg });
            }
        }
    }
    rows
}

/// Build `waydroid app intent <action> <uri>` args.
#[must_use]
pub fn intent_args(action: &str, uri: &str) -> Vec<String> {
    tracing::info!(action, uri, "apps: intent args");
    vec![
        "app".to_owned(),
        "intent".to_owned(),
        action.to_owned(),
        uri.to_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        let pkg = Pkg("com.example.game".to_owned());
        assert_eq!(
            install_args("/tmp/game.apk"),
            vec!["app", "install", "/tmp/game.apk"]
        );
        assert_eq!(remove_args(&pkg), vec!["app", "remove", "com.example.game"]);
        assert_eq!(launch_args(&pkg), vec!["app", "launch", "com.example.game"]);
        assert_eq!(list_args(), vec!["app", "list"]);
        assert_eq!(
            intent_args("android.intent.action.VIEW", "https://x"),
            vec!["app", "intent", "android.intent.action.VIEW", "https://x"]
        );
    }

    #[test]
    fn parses_live_list() {
        let out = "Name: Clash Royale\npackageName: com.supercell.clashroyale\ncategories:\n\tandroid.intent.category.LAUNCHER\nName: Magisk Delta\npackageName: io.github.huskydg.magisk\ncategories:\n\tandroid.intent.category.LAUNCHER\nName: Broken\ncategories:\n\tandroid.intent.category.INFO\n";
        let rows = parse_app_list(out);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "Clash Royale");
        assert_eq!(rows[0].pkg, "com.supercell.clashroyale");
        assert_eq!(rows[1].pkg, "io.github.huskydg.magisk");
    }

    #[test]
    fn skips_empty() {
        assert!(parse_app_list("").is_empty());
        assert!(parse_app_list("Name: \npackageName: \n").is_empty());
    }

    #[test]
    fn launcher_shapes() {
        let pkg = Pkg("com.tencent.ig".to_owned());
        assert_eq!(
            resolve_launcher_args(&pkg),
            vec![
                "shell",
                "cmd",
                "package",
                "resolve-activity",
                "--brief",
                "-a",
                "android.intent.action.MAIN",
                "-c",
                "android.intent.category.LAUNCHER",
                "com.tencent.ig"
            ]
        );
        assert_eq!(
            launcher_start_args("com.tencent.ig/com.epicgames.ue4.SplashActivity"),
            vec![
                "shell",
                "am",
                "start",
                "-a",
                "android.intent.action.MAIN",
                "-c",
                "android.intent.category.LAUNCHER",
                "-n",
                "com.tencent.ig/com.epicgames.ue4.SplashActivity"
            ]
        );
    }

    #[test]
    fn parses_live_resolve() {
        let out = "priority=0 preferredOrder=0 match=0x108000 specificIndex=-1 isDefault=false\ncom.tencent.ig/com.epicgames.ue4.SplashActivity\n";
        assert_eq!(
            parse_launcher_component(out).as_deref(),
            Some("com.tencent.ig/com.epicgames.ue4.SplashActivity")
        );
        assert_eq!(parse_launcher_component(""), None);
        assert_eq!(
            parse_launcher_component("priority=0 preferredOrder=0\n"),
            None
        );
    }
}
