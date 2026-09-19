//! Frame + coord goldens. LE per `ANDROID_SOCKET_PROTOCOL.md:139`.
#![deny(missing_docs)]

use wd_core::RelPos;
use wd_inject::{cancel_frame, down_frame, move_frame, ping_frame, scale, up_frame};

#[test]
fn frames_layout() {
    assert_eq!(
        down_frame(3, 960, 540),
        [0x00, 3, 0xC0, 0x03, 0x00, 0x00, 0x1C, 0x02, 0x00, 0x00]
    );
    let m = move_frame(3, 960, 540);
    assert_eq!((m[0], m[1]), (0x01, 3));
    assert_eq!(up_frame(3), [0x02, 3]);
    assert_eq!(cancel_frame(), [0x03]);
    assert_eq!(ping_frame(), [0x7f]);
}

#[test]
fn scale_clamps() {
    assert_eq!(scale(RelPos { x: 0.5, y: 0.5 }, 1920, 1080), (960, 540));
    assert_eq!(scale(RelPos { x: 9.0, y: -9.0 }, 1920, 1080), (1919, 0));
    assert_eq!(scale(RelPos { x: 0.158, y: 0.789 }, 1920, 1080), (303, 852));
}
