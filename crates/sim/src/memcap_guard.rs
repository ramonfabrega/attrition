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
//!
//! What `ps rss` does *not* count is a page the host has compressed or
//! swapped, so the script counts the larger of it and the physical
//! footprint (gate-4192: this fixture's idle child read under its cap for
//! 120 s; `tools/memcap.sh` has the evidence). A red here prints the
//! script's own samples (`MEMCAP_TRACE`), both processes' state and the
//! host's memory state, so the next one is a diagnosis rather than a re-run.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

/// One command's output for a red's report, or why it gave none.
fn shown(program: &str, args: &[&str]) -> String {
    match Command::new(program).args(args).output() {
        Ok(out) => format!(
            "$ {program} {}\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("$ {program} {}: {e}\n", args.join(" ")),
    }
}

/// What a red says beyond "never fired": memcap.sh's every sample, the child
/// and the script as `ps` sees them, the child's footprint, and how hard the
/// host was squeezing its memory. Taken before anything is killed.
fn diagnosis(trace: &Path, child: Option<&str>, guard: u32) -> String {
    let guard = guard.to_string();
    let pids = child.map_or(guard.clone(), |c| format!("{c},{guard}"));
    let mut text = format!(
        "\n--- memcap.sh's samples (MEMCAP_TRACE, KiB) ---\n{}",
        std::fs::read_to_string(trace).unwrap_or_else(|e| format!("no trace: {e}\n"))
    );
    text += "--- the child and memcap.sh ---\n";
    text += &shown(
        "ps",
        &[
            "-o",
            "pid,ppid,stat,nice,rss,vsz,%cpu,time,wchan,command",
            "-p",
            &pids,
        ],
    );
    if let Some(child) = child {
        text += &shown("footprint", &["-f", "bytes", "--noCategories", "-p", child]);
    }
    text += "--- the host ---\n";
    text += &shown(
        "sysctl",
        &[
            "vm.swapusage",
            "kern.memorystatus_vm_pressure_level",
            "vm.loadavg",
        ],
    );
    text += &shown("vm_stat", &[]);
    text
}

/// A process that climbs past a 1 GiB cap is dead within a few polls, and
/// the exit code is 137 with the reason on stderr.
#[test]
fn memcap_kills_a_process_past_the_cap_with_137() {
    let start = Instant::now();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let trace = std::env::temp_dir().join(format!(
        "attrition-memcap-trace-{}-{stamp}",
        std::process::id()
    ));
    let mut guard = Command::new("zsh")
        .env("MEMCAP_TRACE", &trace)
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
            let report = diagnosis(&trace, pid.as_deref(), guard.id());
            // The child first, so its pipes close and the readers finish.
            if let Some(pid) = &pid {
                let _ = Command::new("kill").args(["-9", pid]).status();
            }
            let _ = guard.kill();
            let _ = guard.wait();
            let _ = std::fs::remove_file(&trace);
            panic!(
                "{why}.\n--- memcap.sh's stderr ---\n{}{report}",
                stderr.join().unwrap_or_default()
            );
        }
    };
    let stderr = stderr.join().unwrap_or_default();
    let samples = std::fs::read_to_string(&trace).unwrap_or_default();
    let _ = std::fs::remove_file(&trace);
    assert_eq!(
        status.code(),
        Some(137),
        "memcap.sh did not kill a 1,200 MiB process under a 1 GiB cap: {stderr}"
    );
    assert!(
        stderr.contains("over the 1 GiB ceiling"),
        "memcap.sh killed without saying why: {stderr}"
    );
    assert!(
        samples.contains(" footprint ") && samples.contains("over the cap: killing"),
        "memcap.sh's trace did not record its samples and its kill: {samples}"
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
