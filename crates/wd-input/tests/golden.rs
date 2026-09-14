//! Goldens on 8 phantom profiles + Qt import roundtrips.
//! Refs: `.devdocs/phantom/profiles/*.json`, `.plans/03-input-keymap.md:48`.
#![deny(missing_docs)]

use std::path::PathBuf;

use wd_input::{Screen, audit, audit_screen, dup_keys, import_qt, load, slot_clashes, validate};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/keymap")
}

const GOLDENS: &[&str] = &[
    "pubg.json",
    "pubg-small.json",
    "genshin.json",
    "efootball-template.json",
    "temple-run.json",
    "subway-surfers.json",
    "asphalt8.json",
    "asphalt9.json",
];

#[test]
fn goldens_load_clean() {
    for name in GOLDENS {
        let p = load(&dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(p.version, 1, "{name}");
        assert!(validate(&p).is_ok(), "{name}");
        let issues = audit_screen(&p, None);
        assert!(issues.is_empty(), "{name}: {issues:?}");
        let report = audit(&p);
        assert!(report.issues.is_empty(), "{name}: {:?}", report.issues);
        assert!(!report.slots.is_empty(), "{name}");
    }
}

#[test]
fn pubg_shape() {
    let p = load(&dir().join("pubg.json")).expect("pubg");
    assert_eq!(p.nodes.len(), 21);
    assert_eq!(p.name, "PUBG Mobile");
}

#[test]
fn dup_slot_version_checks() {
    let p = load(&dir().join("pubg.json")).expect("pubg");
    assert!(dup_keys(&p).iter().any(|i| i.contains("dup key")));
    assert!(slot_clashes(&p).is_empty());
    let mut bad = p.clone();
    bad.version = 99;
    assert!(validate(&bad).is_err());
    assert!(
        audit_screen(&bad, None)
            .iter()
            .any(|i| i.contains("version"))
    );
    bad = p.clone();
    bad.screen = None;
    assert!(validate(&bad).is_err());
    let mut off = p;
    if let wd_input::Node::Tap(t) = &mut off.nodes[5] {
        t.pos.x = 9.0;
    }
    let report = audit(&off);
    assert!(report.issues.iter().any(|i| i.contains("off-screen")));
}

#[test]
fn qt_imports() {
    let raw = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.devdocs/QtScrcpy/keymap/gameforpeace.json"),
    )
    .expect("gameforpeace");
    let p = import_qt(
        "gameforpeace",
        &raw,
        Screen {
            width: 1920,
            height: 1080,
        },
    )
    .expect("import gameforpeace");
    assert!(
        p.nodes
            .iter()
            .any(|n| matches!(n, wd_input::Node::Joystick { .. }))
    );
    assert!(
        p.nodes
            .iter()
            .any(|n| matches!(n, wd_input::Node::Aim { .. }))
    );
    assert!(
        p.nodes
            .iter()
            .any(|n| matches!(n, wd_input::Node::LayerShift { .. }))
    );
    assert!(validate(&p).is_ok());

    let raw2 = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.devdocs/ScrcpyKeyMapper/examples/pubg.json"),
    )
    .expect("mapper pubg");
    let p2 = import_qt(
        "mapper-pubg",
        &raw2,
        Screen {
            width: 2336,
            height: 1080,
        },
    )
    .expect("import mapper pubg");
    assert!(
        p2.nodes
            .iter()
            .any(|n| matches!(n, wd_input::Node::Drag { .. }))
    );
    assert!(
        p2.nodes
            .iter()
            .any(|n| matches!(n, wd_input::Node::Macro { .. }))
    );
    assert!(validate(&p2).is_ok());
}
