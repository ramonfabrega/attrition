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

use render::clock::{CLOCK, Ceilings, Clock};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

fn gamelog_dir() -> PathBuf {
    std::env::var_os("RON_GAMELOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs")
        })
}

/// How long a broken first session waits for a loaded host to fall under
/// its cores before the second is measured anyway: 600 s, set by the pass
/// of 2026-11-12, when four workers' gates held the host at 46 to 51 on 16
/// cores for the better part of an hour and both sessions, 17 s apart,
/// broke p99 (50.04 ms, then 73.75 ms) on a client an A/B read at 5.5.
const QUIET_WITHIN: Duration = Duration::from_secs(600);
/// The load average is a one-minute mean; five seconds between reads is
/// fine-grained against it.
const QUIET_POLL: Duration = Duration::from_secs(5);

/// The one-minute load average from `sysctl -n vm.loadavg`'s line
/// (`{ 7.91 8.99 12.66 }`, macOS) or `/proc/loadavg`'s (`0.52 0.61 0.70
/// 1/200 1234`, Linux).
fn one_minute_load(text: &str) -> Option<f64> {
    let first = text.trim().trim_start_matches('{').split_whitespace().next()?;
    first.parse().ok()
}

/// The host's one-minute load, or `None` where neither source reads.
fn host_load() -> Option<f64> {
    let sysctl = Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| one_minute_load(&String::from_utf8_lossy(&out.stdout)));
    sysctl.or_else(|| {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|text| one_minute_load(&text))
    })
}

fn cores() -> usize {
    std::thread::available_parallelism().map_or(0, |n| n.get())
}

fn load_words(load: Option<f64>, cores: usize) -> String {
    match load {
        Some(load) => format!("{load:.2} on {cores} cores"),
        None => format!("unread on {cores} cores"),
    }
}

fn loaded(load: Option<f64>, cores: usize) -> bool {
    load.is_some_and(|load| load > cores as f64)
}

/// What waiting for the host came to.
#[derive(Debug, PartialEq)]
enum Wait {
    /// The host was under its cores (or its load unreadable): no wait.
    None,
    /// It fell under after this long, at this load.
    Quiet { waited: Duration, load: f64 },
    /// It stayed over its cores for the whole bound; the last load read.
    StayedLoaded { load: f64 },
}

/// Wait, polling `read`, until the load is at or under `cores`, for at most
/// `within`. `sleep` is injected so the unit tests run on a counted clock.
fn wait_for_quiet(
    mut read: impl FnMut() -> Option<f64>,
    cores: usize,
    within: Duration,
    poll: Duration,
    mut sleep: impl FnMut(Duration),
) -> Wait {
    let mut load = read();
    if !loaded(load, cores) {
        return Wait::None;
    }
    let mut waited = Duration::ZERO;
    while waited < within {
        sleep(poll);
        waited += poll;
        match read() {
            // Unreadable mid-wait: keep the last load read and go on.
            None => {}
            Some(now) if now <= cores as f64 => return Wait::Quiet { waited, load: now },
            now => load = now,
        }
    }
    Wait::StayedLoaded {
        load: load.unwrap_or_default(),
    }
}

/// The failure, when the second session breaks; `None` when it does not.
fn verdict(
    first: (&[String], Option<f64>),
    second: (&[String], Option<f64>),
    wait: &Wait,
    cores: usize,
) -> Option<String> {
    if second.0.is_empty() {
        return None;
    }
    let sessions = format!(
        "first {} (host load {}); then {} (host load {})",
        first.0.join("; "),
        load_words(first.1, cores),
        second.0.join("; "),
        load_words(second.1, cores)
    );
    Some(match wait {
        Wait::StayedLoaded { load } => format!(
            "the host stayed loaded: {load:.2} on {cores} cores for {} s; unmeasured, re-run \
             (both sessions broke the clock's ceilings on a host over its cores, which says \
             nothing about the client): {sessions}",
            QUIET_WITHIN.as_secs()
        ),
        _ => format!(
            "the clock breaks its ceilings twice running (render::clock::CLOCK): {sessions}"
        ),
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
    let ceilings: Ceilings = CLOCK;
    let cores = cores();
    // One session of the client, what of the ceilings its clock breaks,
    // and the host's load as it ended (a one-minute mean, so it spans it).
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
            .args(["--frame", &ceilings.from_frame.to_string()])
            .args(["--frames", &ceilings.frames.to_string()])
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
        let load = host_load();
        eprintln!(
            "CLOCK {}: {clock:?} (host load {})",
            ceilings.game,
            load_words(load, cores)
        );
        (ceilings.broken_by(&clock), load)
    };
    eprintln!("CLOCK pinned: {ceilings:?}");
    // **A break is measured twice before it is believed** (the pass of
    // 2026-11-12, `docs/RENDERER.md` §3.11): the host is shared, and one
    // gate read p99 75 ms where the session alone reads under 6. A stall of
    // the host breaks one session; a slower client breaks both. And a host
    // that stays loaded breaks both too, so the second waits for the host
    // to fall under its cores (the same pass, after a gate where both
    // sessions broke 17 s apart at load 46 to 51 on 16).
    let (first, first_load) = session();
    if first.is_empty() {
        return;
    }
    eprintln!(
        "clock: the first session broke its ceilings ({}) at host load {}; measuring once more",
        first.join("; "),
        load_words(first_load, cores)
    );
    let wait = wait_for_quiet(host_load, cores, QUIET_WITHIN, QUIET_POLL, std::thread::sleep);
    match &wait {
        Wait::None => {}
        Wait::Quiet { waited, load } => eprintln!(
            "clock: waited {} s for the host to fall to {load:.2} on {cores} cores",
            waited.as_secs()
        ),
        Wait::StayedLoaded { load } => eprintln!(
            "clock: the host stayed loaded ({load:.2} on {cores} cores) for {} s; measuring anyway",
            QUIET_WITHIN.as_secs()
        ),
    }
    let (second, second_load) = session();
    if let Some(failure) = verdict(
        (&first, first_load),
        (&second, second_load),
        &wait,
        cores,
    ) {
        panic!("{failure}");
    }
}

#[test]
fn the_load_reader_reads_sysctl_and_proc_lines() {
    assert_eq!(one_minute_load("{ 7.91 8.99 12.66 }\n"), Some(7.91));
    assert_eq!(one_minute_load("{ 50.90 46.02 31.70 }"), Some(50.9));
    assert_eq!(one_minute_load("0.52 0.61 0.70 1/200 1234\n"), Some(0.52));
    assert_eq!(one_minute_load(""), None);
    assert_eq!(one_minute_load("{ }"), None);
    assert_eq!(one_minute_load("sysctl: unknown oid"), None);
}

#[test]
fn a_quiet_host_is_not_waited_on() {
    let mut slept = Duration::ZERO;
    let wait = wait_for_quiet(|| Some(4.0), 16, QUIET_WITHIN, QUIET_POLL, |d| slept += d);
    assert_eq!(wait, Wait::None);
    assert_eq!(slept, Duration::ZERO);
    // Unreadable is not loaded: the two-session behaviour stands as it was.
    let wait = wait_for_quiet(|| None, 16, QUIET_WITHIN, QUIET_POLL, |d| slept += d);
    assert_eq!(wait, Wait::None);
    assert_eq!(slept, Duration::ZERO);
}

#[test]
fn a_loaded_host_is_waited_on_until_it_falls() {
    let mut loads = [51.0, 40.0, 20.0, 15.5].into_iter();
    let mut slept = Duration::ZERO;
    let wait = wait_for_quiet(
        || loads.next(),
        16,
        QUIET_WITHIN,
        QUIET_POLL,
        |d| slept += d,
    );
    assert_eq!(
        wait,
        Wait::Quiet {
            waited: QUIET_POLL * 3,
            load: 15.5
        }
    );
    assert_eq!(slept, QUIET_POLL * 3);
}

#[test]
fn a_host_that_stays_loaded_is_named_not_the_client() {
    // A reader that never falls, on a counted clock: the wait runs out at
    // its bound, and a second break under it says so in a driver's words.
    let mut slept = Duration::ZERO;
    let wait = wait_for_quiet(|| Some(50.9), 16, QUIET_WITHIN, QUIET_POLL, |d| slept += d);
    assert_eq!(wait, Wait::StayedLoaded { load: 50.9 });
    assert!(slept >= QUIET_WITHIN && slept < QUIET_WITHIN + QUIET_POLL);
    let first = vec!["p99 frame 50.04 ms over its ceiling 20".to_string()];
    let second = vec!["p99 frame 73.75 ms over its ceiling 20".to_string()];
    let message = verdict((&first, Some(51.2)), (&second, Some(50.9)), &wait, 16).unwrap();
    assert!(
        message.contains("the host stayed loaded: 50.90 on 16 cores for 600 s; unmeasured, re-run"),
        "{message}"
    );
    assert!(message.contains("50.04 ms") && message.contains("73.75 ms"), "{message}");
    assert!(message.contains("51.20 on 16 cores"), "{message}");
    // A second session under every ceiling passes, whatever the host did.
    assert_eq!(verdict((&first, Some(51.2)), (&[], Some(50.9)), &wait, 16), None);
    // A quiet host's second break is the client's, and is not called the host's.
    let message = verdict((&first, Some(3.0)), (&second, Some(2.5)), &Wait::None, 16).unwrap();
    assert!(!message.contains("stayed loaded"), "{message}");
    assert!(message.contains("2.50 on 16 cores"), "{message}");
}
