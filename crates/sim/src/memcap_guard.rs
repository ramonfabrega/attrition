//! `tools/memcap.sh` has teeth, and this is the fixture that proves it
//! (parked 251, the eighth pass).
//!
//! The script is the mechanical ceiling every gate runs under: it polls the
//! command's resident set and every descendant's every two seconds and
//! `kill -9`s the tree on the first sample over the cap, exiting 137 so a
//! gate that reads the code cannot mistake an OOM kill for a test failure.
//! It was written after a worker's release run grew to 27.6 GB and took the
//! machine down, and until now nothing had ever made it fire on purpose —
//! which, for a guard, is the same as not knowing whether it works.
//!
//! Two doors stay open and are accepted, named here so nobody re-derives
//! them: the cap is the caller's first argument, so a caller can pass a
//! number the machine cannot honour; and `ps rss` over-counts a shared
//! mapping in every process that maps it, which errs toward killing early,
//! while a two-second poll under-reports a sawtooth, which errs late by at
//! most two seconds of growth.

use std::process::Command;
use std::time::Instant;

fn memcap() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/memcap.sh").to_string()
}

/// A process that climbs past a 1 GiB cap is dead within a few polls, and
/// the exit code is 137 with the reason on stderr.
#[test]
fn memcap_kills_a_process_past_the_cap_with_137() {
    let start = Instant::now();
    let out = Command::new("zsh")
        .arg(memcap())
        .arg("1")
        .arg("python3")
        .arg("-c")
        // 1,200 MiB touched — `b'x' * n` writes every byte, where a zeroed
        // buffer would leave its pages unmapped and its RSS at nothing.
        .arg("b = b'x' * (1200 * 2**20)\nimport time\ntime.sleep(60)\n")
        .output()
        .expect("zsh tools/memcap.sh");
    let secs = start.elapsed().as_secs();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(137),
        "memcap.sh did not kill a 1,200 MiB process under a 1 GiB cap: {stderr}"
    );
    assert!(
        stderr.contains("over the 1 GiB ceiling"),
        "memcap.sh killed without saying why: {stderr}"
    );
    assert!(
        secs < 30,
        "memcap.sh took {secs} s to fire; the poll is two seconds and the climb is instant"
    );
}

/// Under the cap, the command's own exit status comes through untouched.
#[test]
fn memcap_passes_the_command_s_own_exit_status_through() {
    let out = Command::new("zsh")
        .arg(memcap())
        .arg("1")
        .arg("sh")
        .arg("-c")
        .arg("exit 3")
        .output()
        .expect("zsh tools/memcap.sh");
    assert_eq!(
        out.status.code(),
        Some(3),
        "memcap.sh rewrote the command's exit status: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
