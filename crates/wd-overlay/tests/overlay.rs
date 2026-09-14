//! Overlay gates: modes, rects, hotkeys, OSD, guards, bridge.
#![deny(missing_docs)]

use wd_overlay::{
    EditorCanvas, EditorMode, HotkeyAction, OsdState, OverlayLayout, WidgetKind, action_for,
    audit_profile, canvas_binds, canvas_keymap, capturable, dup_free, load_profile,
    normalize_capture, switch_on_key, validate_profile,
};

#[test]
fn f1_toggles_f12_transparency() {
    assert!(EditorMode::is_switch_key("F1"));
    assert!(EditorMode::is_transparency_key("F12"));
    assert_eq!(switch_on_key(EditorMode::Edit, "F1"), EditorMode::Mapping);
    assert_eq!(switch_on_key(EditorMode::Mapping, "F1"), EditorMode::Edit);
    assert!(!EditorMode::Edit.wants_grab());
    assert!(EditorMode::Mapping.wants_grab());
}

#[test]
fn canvas_add_drag_resize_capture() {
    let mut c = EditorCanvas::new((1920, 1080));
    c.add("tap1", WidgetKind::Tap, 0.5, 0.5);
    c.drag_selected(0.1, 0.0);
    let w = &c.widgets[0];
    assert!((w.x - 0.6).abs() < f32::EPSILON);
    c.resize_selected(0.05, 0.05);
    assert!(c.widgets[0].w > 0.0);
    assert!(c.capture_selected("Ctrl+A"));
    assert_eq!(c.widgets[0].key, "LeftCtrl+A");
    assert!(!c.capture_selected("Nope+Bad+Key+Long"));
    assert!(!c.wants_grab());
    let _ = c.on_key("F1", false);
    assert_eq!(c.mode, EditorMode::Mapping);
    assert!(c.wants_grab());
    c.drag_selected(0.5, 0.5);
    assert!((c.widgets[0].x - 0.6).abs() < f32::EPSILON);
}

#[test]
fn capture_spellings() {
    assert!(capturable("a"));
    assert!(capturable("F1"));
    assert!(capturable("MouseLeft"));
    assert!(capturable("Ctrl+Shift+P"));
    assert_eq!(normalize_capture("ctrl+a").as_deref(), Some("LeftCtrl+A"));
    assert!(normalize_capture("BogusKey").is_none());
}

#[test]
fn hotkeys_phantom_defaults() {
    assert_eq!(action_for("F1", false), Some(HotkeyAction::RouteMouse));
    assert_eq!(action_for("F8", false), Some(HotkeyAction::Capture));
    assert_eq!(action_for("F9", false), Some(HotkeyAction::Pause));
    assert_eq!(action_for("F10", false), Some(HotkeyAction::Preview));
    assert_eq!(action_for("F2", false), Some(HotkeyAction::Shutdown));
    assert_eq!(action_for("p", true), Some(HotkeyAction::PauseXt));
    assert_eq!(action_for("e", true), Some(HotkeyAction::EditorXt));
    assert_eq!(action_for("s", true), Some(HotkeyAction::SaveXt));
    assert_eq!(action_for("q", false), None);
    assert!(dup_free(&["F1", "F8", "F9", "F10", "F2"]));
    assert!(!dup_free(&["F1", "f1"]));
}

#[test]
fn layout_roundtrip_scales() {
    let mut c = EditorCanvas::new((1000, 500));
    c.add("dpad", WidgetKind::Dpad, 0.25, 0.5);
    let doc = OverlayLayout::from_rects(&c.widgets, (1000, 500));
    assert_eq!(doc.screen, (1000, 500));
    let back = doc.to_rects();
    assert!((back[0].x - 0.25).abs() < 0.001);
    assert!((back[0].y - 0.5).abs() < 0.001);
}

#[test]
fn osd_pause_toast_tick() {
    let mut o = OsdState::default();
    o.set_paused(true);
    assert!(o.paused);
    o.toast("saved");
    assert_eq!(o.toast, "saved");
    for _ in 0..120 {
        o.tick();
    }
    assert!(o.toast.is_empty());
    o.set_minimal(true);
    assert!(o.minimal);
}

#[test]
fn key_fires_shared_viewmodel() {
    let mut c = EditorCanvas::new((1920, 1080));
    c.add("fire", WidgetKind::Fire, 0.8, 0.8);
    assert!(c.capture_selected("MouseLeft"));
    let vm = canvas_keymap(&c, "pubg");
    assert_eq!(vm.profile, "pubg");
    assert_eq!(vm.fire_key, "MouseLeft");
    assert!(canvas_binds(&c).is_empty());
}

#[test]
fn bridge_reuses_wd_input() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../profiles/keymap/pubg.json");
    let p = load_profile(&dir).expect("pubg loads");
    assert!(validate_profile(&p).is_ok());
    let report = audit_profile(&p);
    assert!(!report.slots.is_empty());
}
