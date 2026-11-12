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

use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn memcap() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/memcap.sh").to_string()
}

/// The child: say its pid, touch 1,200 MiB (`b'x' * n` writes every byte,
/// where a zeroed buffer would leave its pages unmapped and its RSS at
/// nothing), say it holds them, and hold them until killed. A day is far
/// longer than any gate, so a memcap that never fires is caught by the
/// bounds below rather than by the child leaving on its own. The first
/// version slept 60 s and bounded the whole run at 30: on a lane's gate at
/// load 50.9 on 16 cores the child had touched 805 MiB when its sleep ran
/// out, so memcap never saw it past the cap (the pass of 2026-11-12).
const CHILD: &str = "import os, time\n\
print('pid', os.getpid(), flush=True)\n\
b = b'x' * (1200 * 2**20)\n\
print('held', flush=True)\n\
time.sleep(86400)\n";

/// From the moment the child holds its 1,200 MiB, memcap's next poll is two
/// seconds away (`tools/memcap.sh`'s `sleep 2`). Two minutes is sixty polls;
/// a memcap that polls at all fires inside it. The clock starts at `held`,
/// so a slow allocation cannot spend it.
const FIRES_WITHIN: Duration = Duration::from_secs(120);

/// The allocation's own ceiling, so a child that never reaches `held` still
/// fails in bounded time. Alone it takes about a second; at load 50.9 it
/// touched 805 MiB in 60 s (the pass of 2026-11-12), which puts the whole
/// 1,200 MiB near 90 s.
const HOLDS_WITHIN: Duration = Duration::from_secs(600);

/// A process that climbs past a 1 GiB cap is dead within a few polls, and
/// the exit code is 137 with the reason on stderr.
#[test]
fn memcap_kills_a_process_past_the_cap_with_137() {
    let start = Instant::now();
    let mut guard = Command::new("zsh")
        .arg(memcap())
        .arg("1")
        .arg("python3")
        .arg("-c")
        .arg(CHILD)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zsh tools/memcap.sh");
    let (lines_tx, lines) = mpsc::channel();
    let stdout = guard.stdout.take().unwrap();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if lines_tx.send(line).is_err() {
                break;
            }
        }
    });
    let mut stderr_pipe = guard.stderr.take().unwrap();
    let stderr = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr_pipe.read_to_string(&mut text);
        text
    });
    let mut pid: Option<String> = None;
    let mut held: Option<Instant> = None;
    let status = loop {
        if let Some(status) = guard.try_wait().expect("wait on memcap.sh") {
            break status;
        }
        match lines.recv_timeout(Duration::from_millis(200)) {
            Ok(line) if line.starts_with("pid ") => pid = Some(line[4..].trim().to_string()),
            Ok(line) if line == "held" => held = Some(Instant::now()),
            _ => {}
        }
        let late = match held {
            Some(at) if at.elapsed() > FIRES_WITHIN => Some(format!(
                "the child held 1,200 MiB under a 1 GiB cap for {} s and memcap.sh never killed it",
                FIRES_WITHIN.as_secs()
            )),
            None if start.elapsed() > HOLDS_WITHIN => Some(format!(
                "the child had not touched its 1,200 MiB after {} s and memcap.sh had not killed it",
                HOLDS_WITHIN.as_secs()
            )),
            _ => None,
        };
        if let Some(why) = late {
            // The child first, so its pipes close and the readers finish.
            if let Some(pid) = &pid {
                let _ = Command::new("kill").args(["-9", pid]).status();
            }
            let _ = guard.kill();
            let _ = guard.wait();
            panic!("{why}: {}", stderr.join().unwrap_or_default());
        }
    };
    let stderr = stderr.join().unwrap_or_default();
    assert_eq!(
        status.code(),
        Some(137),
        "memcap.sh did not kill a 1,200 MiB process under a 1 GiB cap: {stderr}"
    );
    assert!(
        stderr.contains("over the 1 GiB ceiling"),
        "memcap.sh killed without saying why: {stderr}"
    );
    eprintln!(
        "memcap fired {} s after launch{}",
        start.elapsed().as_secs(),
        held.map_or(
            String::from(", before the child held its 1,200 MiB"),
            |at| format!(
                ", {} ms after the child held its 1,200 MiB",
                at.elapsed().as_millis()
            )
        )
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
