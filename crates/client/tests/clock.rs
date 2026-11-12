//! **The clock, under its ceilings** (item 4161; `docs/RENDERER.md` §3.11;
//! `render::clock::CLOCK`). The client binary plays Great Lakes from our
//! own start — `--lobby`, the minted script's lobby built from its choices
//! — off-screen for the pinned frames, writes its session trace, and the
//! clock read from that trace stands under every ceiling the queue's
//! `Clock:` line prints, its slowest frame the one the line names.
//!
//! The session is a process of its own, so the milliseconds from launch
//! and the resident set are the client's, not this harness's. It needs
//! the install (`RON_INSTALL`) and the captures' directory (the pair's
//! start dump names the ground's tileset; the lobby dump its choices); a
//! machine without them says `skipping:` and passes vacuously.

use render::clock::{CLOCK, Clock};
use std::path::PathBuf;
use std::process::Command;

fn gamelog_dir() -> PathBuf {
    std::env::var_os("RON_GAMELOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
        })
}

#[test]
fn the_great_lakes_lobby_plays_under_the_clock_s_ceilings() {
    let Some(install) = std::env::var_os("RON_INSTALL").map(PathBuf::from) else {
        eprintln!("skipping: RON_INSTALL is not set");
        return;
    };
    let logs = gamelog_dir();
    for need in [
        logs.join(rondata::script::LOBBY_DUMP),
        logs.join("gamelog-run1428-greatlakes-aztec-modern-start.txt"),
    ] {
        if !need.exists() {
            eprintln!("skipping: {} is not on this machine", need.display());
            return;
        }
    }
    // One session of the client, and what of the ceilings its clock breaks.
    let session = || {
        let dir = std::env::temp_dir().join(format!("att-client-clock-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let trace = dir.join("session.json");
        let out = Command::new(env!("CARGO_BIN_EXE_client"))
            .arg("--install")
            .arg(&install)
            .arg("--logs")
            .arg(&logs)
            .args(["--lobby", "--headless", "--play"])
            .args(["--frame", &CLOCK.from_frame.to_string()])
            .args(["--frames", &CLOCK.frames.to_string()])
            .arg("--trace")
            .arg(&trace)
            .output()
            .expect("the client runs");
        let text = std::fs::read_to_string(&trace).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            out.status.success(),
            "client exited {:?}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
        let clock = Clock::read(&text).expect("the trace holds a `frame zero` mark");
        eprintln!("CLOCK {}: {clock:?}", CLOCK.game);
        CLOCK.broken_by(&clock)
    };
    eprintln!("CLOCK pinned: {CLOCK:?}");
    // **A break is measured twice before it is believed** (the pass of
    // 2026-11-12, `docs/RENDERER.md` §3.11): the host is shared, and one
    // gate read p99 75 ms where the session alone reads under 6. A stall of
    // the host breaks one session; a slower client breaks both.
    let first = session();
    if first.is_empty() {
        return;
    }
    eprintln!(
        "clock: the first session broke its ceilings ({}); measuring once more",
        first.join("; ")
    );
    let second = session();
    assert!(
        second.is_empty(),
        "the clock breaks its ceilings twice running (render::clock::CLOCK): first {}; then {}",
        first.join("; "),
        second.join("; ")
    );
}
