//! **The session trace** (`docs/DECISIONS.md` entry 114 §4; `docs/RENDERER.md`
//! §3.6): the client's own record of a session from launch to quit, in the
//! Trace Event Format — the JSON array Chrome's `about:tracing` and
//! Perfetto open as they stand — so the whole flow can be looked at in a
//! tool nobody here wrote, and the clock is read from the file
//! ([`Clock::read`], `render::clock`) rather than from a stopwatch beside
//! it. A drawn frame's phases — `step`, `scene`, `ground`, `gpu`, `wait` —
//! are spans nested inside its `frame` span, so the slowest frame is read
//! with its cause.
//!
//! Three kinds of event are written:
//! - a **span** (`"ph":"X"`, with `dur`) for each phase and each drawn frame;
//! - an **instant** (`"ph":"i"`) for a mark — `frame zero`, `quit`;
//! - a **counter** (`"ph":"C"`) for the resident set, `rss_kib`.
//!
//! The resident set is sampled by `ps -o rss=` on a thread every 100 ms: the
//! workspace forbids `unsafe`, so `getrusage` is out of reach, and the
//! sampled peak is a floor on the true one (`docs/RENDERER.md` §3.6).
//! Times are microseconds from the session's start.

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// One event, kept until the session writes the file.
#[derive(Clone, Debug)]
enum Event {
    Span { name: String, ts: u64, dur: u64 },
    Mark { name: String, ts: u64 },
    Rss { ts: u64, kib: u64 },
}

/// The session's recorder. Cloning shares it.
#[derive(Clone)]
pub struct Session {
    start: Instant,
    events: Arc<Mutex<Vec<Event>>>,
    sampling: Arc<Mutex<bool>>,
}

impl Session {
    /// A session starting now, with the resident-set sampler running.
    pub fn start() -> Self {
        let s = Session {
            start: Instant::now(),
            events: Arc::new(Mutex::new(Vec::new())),
            sampling: Arc::new(Mutex::new(true)),
        };
        let sampler = s.clone();
        std::thread::spawn(move || {
            while *sampler.sampling.lock().unwrap() {
                sampler.sample_rss();
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        s
    }

    fn now(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    fn push(&self, e: Event) {
        self.events.lock().unwrap().push(e);
    }

    /// A span from `since` to now.
    pub fn span(&self, name: &str, since: Instant) {
        let ts = since.duration_since(self.start).as_micros() as u64;
        let dur = since.elapsed().as_micros() as u64;
        self.push(Event::Span {
            name: name.into(),
            ts,
            dur,
        });
    }

    /// Run `f` as a span.
    pub fn phase<R>(&self, name: &str, f: impl FnOnce() -> R) -> R {
        let t = Instant::now();
        let r = f();
        self.span(name, t);
        r
    }

    /// A mark at now.
    pub fn mark(&self, name: &str) {
        let ts = self.now();
        self.push(Event::Mark {
            name: name.into(),
            ts,
        });
    }

    /// One reading of this process's resident set, in KiB.
    pub fn sample_rss(&self) {
        let out = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output();
        if let Some(kib) = out
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse().ok())
        {
            let ts = self.now();
            self.push(Event::Rss { ts, kib });
        }
    }

    /// Stop the sampler, take a last reading, mark the quit and write the
    /// file.
    pub fn finish(&self, path: &std::path::Path) -> std::io::Result<()> {
        *self.sampling.lock().unwrap() = false;
        self.sample_rss();
        self.mark("quit");
        let events = self.events.lock().unwrap().clone();
        let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
        writeln!(f, "[")?;
        let pid = std::process::id();
        let mut first = true;
        for e in events {
            let line = match e {
                Event::Span { name, ts, dur } => format!(
                    r#"{{"name":{},"cat":"client","ph":"X","ts":{ts},"dur":{dur},"pid":{pid},"tid":1}}"#,
                    json_string(&name)
                ),
                Event::Mark { name, ts } => format!(
                    r#"{{"name":{},"cat":"client","ph":"i","s":"p","ts":{ts},"pid":{pid},"tid":1}}"#,
                    json_string(&name)
                ),
                Event::Rss { ts, kib } => format!(
                    r#"{{"name":"rss_kib","cat":"client","ph":"C","ts":{ts},"pid":{pid},"tid":1,"args":{{"rss_kib":{kib}}}}}"#
                ),
            };
            if !first {
                writeln!(f, ",")?;
            }
            first = false;
            write!(f, "{line}")?;
        }
        writeln!(f, "\n]")?;
        f.flush()
    }
}

fn json_string(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// The clock a trace holds: `render::clock`'s reader, which the queue's
/// `Clock:` line and the ceilings' test share (item 4161).
pub use render::clock::Clock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_trace_reads_back_as_its_clock() {
        let dir = std::env::temp_dir().join(format!("att-client-trace-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.json");
        let s = Session::start();
        s.phase("load", || std::thread::sleep(Duration::from_millis(5)));
        s.mark("frame zero");
        for _ in 0..3 {
            let t = Instant::now();
            std::thread::sleep(Duration::from_millis(2));
            s.span("frame", t);
        }
        s.mark("first pixels");
        s.finish(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(text.starts_with("[\n") && text.trim_end().ends_with(']'));
        assert_eq!(
            text.matches(r#""ph":"X""#).count(),
            4,
            "load and three frames"
        );
        let c = Clock::read(&text).unwrap();
        assert!(c.to_frame_zero_ms >= 5.0, "{c:?}");
        assert!(c.to_first_pixels_ms >= c.to_frame_zero_ms, "{c:?}");
        assert_eq!(c.frames, 3);
        assert!(
            c.p99_frame_ms >= 2.0 && c.p99_frame_ms <= c.max_frame_ms,
            "{c:?}"
        );
        assert!(c.peak_rss_kib > 0, "the last sample is taken at the finish");
    }

    #[test]
    fn a_name_is_escaped() {
        assert_eq!(json_string("a\"b\\c\n"), r#""a\"b\\c\u000a""#);
    }
}
