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

/// Build `waydroid app list` args.
#[must_use]
pub fn list_args() -> Vec<String> {
    tracing::info!("apps: list args");
    vec!["app".to_owned(), "list".to_owned()]
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
}
