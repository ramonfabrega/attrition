//! The camera on a real frame: the game's own camera, `graphsys` and console
//! as run1484's packet holds them (`~/ron-golden/run1484/map-18`, logger
//! frame 676), the original's `MiscAccess::world_to_screen` for every
//! `SUBOBJECT` the dump prints on that frame, against this crate's
//! projection of the camera fields read off the same packet
//! (`tools/render/packet_points.py`, `docs/RENDERER.md` §3.4).
//!
//! The packet and the dump stay outside the repository; with either absent
//! (`RON_CAMERA_PACKET` and `RON_CAMERA_GAMELOG` move them), no install or
//! no `uv`, the test skips and says so.

mod common;

use std::path::PathBuf;

const FRAME: u32 = 676;

/// Points in register on the packet, of every dumped position projected
/// twice (the packet's own 1920×1080 from `graphsys`, and 1024×768). Measured
/// by this test on its first run (item 4018, 2026-11-08): 394 of 394 — 197
/// `SUBOBJECT`s on block 676 — with the packet's `to_screen_matrix`
/// bit-identical to this crate's.
const IN_REGISTER: usize = 394;

fn path(var: &str, default: &str) -> Option<PathBuf> {
    let p = std::env::var(var)
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            let home = std::env::var("HOME").ok()?;
            Some(PathBuf::from(home).join(default))
        })?;
    if p.exists() {
        Some(p)
    } else {
        eprintln!(
            "skipping: {} is not on this machine ({var} moves it)",
            p.display()
        );
        None
    }
}

#[test]
fn the_packets_camera_is_in_register_with_the_original() {
    let Some(install) = common::install() else {
        return;
    };
    let Some(packet) = path(
        "RON_CAMERA_PACKET",
        "ron-golden/run1484/map-18/frame-snapshot.bin",
    ) else {
        return;
    };
    let Some(gamelog) = path(
        "RON_CAMERA_GAMELOG",
        "ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs/gamelog-run1484-eastindies-information-melee-window-660-680.txt",
    ) else {
        return;
    };
    let exe = install.join("riseofnations.exe");
    let Some(rows) = common::oracle(
        "packet_points.py",
        &[&exe, &packet, &gamelog],
        &[FRAME.to_string()],
    ) else {
        return;
    };
    let states = common::parse(&rows);
    assert_eq!(states.len(), 1, "one camera on a packet");
    let s = &states[0];
    assert!(
        s.cam.parallel,
        "the game's camera is the parallel projection"
    );
    let score = common::in_register(&states);
    eprintln!(
        "packet points in register: {} of {}; worst to_screen element {:e} relative",
        score.points, score.of_points, score.worst_matrix
    );
    assert_eq!(
        score.of_points, IN_REGISTER,
        "every dumped position was projected"
    );
    assert_eq!(
        score.points, IN_REGISTER,
        "re-pin IN_REGISTER with its delta in its comment"
    );
}
