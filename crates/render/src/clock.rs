//! **The renderer's clock** (`docs/DECISIONS.md` entry 114 §4;
//! `docs/RENDERER.md` §3.6, §3.11): what a client session costs, read from
//! the session trace the client writes (`client::trace`, the Trace Event
//! Format Perfetto opens), and the ceilings the queue's `Clock:` line
//! prints (item 4161).
//!
//! The reading lives here, in the headless core, so that the client that
//! writes the trace, the test that plays a session under the ceilings
//! (`crates/client/tests/clock.rs`) and the queue's parser
//! (`rondata::diff::floors::clock_line`) read one definition and one pin.

/// **The clock, read from a session trace.**
#[derive(Clone, Debug, PartialEq)]
pub struct Clock {
    /// Launch to the `frame zero` mark: the install read, the game stood
    /// up at frame zero.
    pub to_frame_zero_ms: f64,
    /// Launch to the `first pixels` mark: the first frame drawn.
    pub to_first_pixels_ms: f64,
    /// The drawn frames' spans: how many, and the 99th percentile and the
    /// slowest, in milliseconds.
    pub frames: usize,
    pub p99_frame_ms: f64,
    pub max_frame_ms: f64,
    /// Which drawn frame was the slowest, counted from 1, and its cause:
    /// the longest span inside it, by name, with its milliseconds.
    pub slowest_frame: usize,
    pub slowest_cause: Option<(String, f64)>,
    /// The largest resident-set sample, KiB (a floor on the true peak: the
    /// client samples `ps` every 100 ms).
    pub peak_rss_kib: u64,
}

impl Clock {
    /// Read the clock off a trace the client wrote. It parses only the
    /// shapes `client::trace::Session::finish` writes, one event a line:
    /// a span (`"ph":"X"`) for each phase and each drawn `frame`, with the
    /// phases a frame ran nested inside its span; the marks `frame zero`
    /// and `first pixels`; the `rss_kib` counter. `None` without a `frame
    /// zero` mark.
    pub fn read(text: &str) -> Option<Clock> {
        let field = |line: &str, key: &str| -> Option<f64> {
            let at = line.find(&format!("\"{key}\":"))? + key.len() + 3;
            let rest = &line[at..];
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .unwrap_or(rest.len());
            rest[..end].parse().ok()
        };
        let name = |line: &str| -> Option<String> {
            let at = line.find("\"name\":\"")? + 8;
            let rest = &line[at..];
            Some(rest[..rest.find('"')?].to_string())
        };
        let (mut zero, mut first) = (None, None);
        // (ts, dur) of each frame, in the order drawn; (name, ts, dur) of
        // every other span.
        let mut frames: Vec<(f64, f64)> = Vec::new();
        let mut phases: Vec<(String, f64, f64)> = Vec::new();
        let mut peak = 0u64;
        for line in text.lines() {
            let Some(n) = name(line) else { continue };
            if n == "frame zero" {
                zero = field(line, "ts");
            } else if n == "first pixels" {
                first = field(line, "ts");
            } else if n == "rss_kib" {
                peak = peak.max(field(line, "rss_kib")? as u64);
            } else if line.contains(r#""ph":"X""#)
                && let (Some(ts), Some(dur)) = (field(line, "ts"), field(line, "dur"))
            {
                if n == "frame" {
                    frames.push((ts, dur));
                } else {
                    phases.push((n, ts, dur));
                }
            }
        }
        frames.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (slowest, &(sts, sdur)) = frames
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.1.total_cmp(&b.1.1).then(b.0.cmp(&a.0)))
            .unwrap_or((0, &(0.0, 0.0)));
        let slowest_cause = phases
            .iter()
            .filter(|(_, ts, dur)| *ts >= sts && ts + dur <= sts + sdur)
            .max_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(n, _, dur)| (n.clone(), dur / 1000.0));
        let mut durs: Vec<f64> = frames.iter().map(|f| f.1).collect();
        durs.sort_by(f64::total_cmp);
        let pct = |p: f64| {
            durs.get(((durs.len() as f64 * p).ceil() as usize).saturating_sub(1))
                .copied()
                .unwrap_or(0.0)
        };
        Some(Clock {
            to_frame_zero_ms: zero? / 1000.0,
            to_first_pixels_ms: first.unwrap_or(0.0) / 1000.0,
            frames: durs.len(),
            p99_frame_ms: pct(0.99) / 1000.0,
            max_frame_ms: durs.last().copied().unwrap_or(0.0) / 1000.0,
            slowest_frame: if durs.is_empty() { 0 } else { slowest + 1 },
            slowest_cause,
            peak_rss_kib: peak,
        })
    }
}

/// **The clock's ceilings**: the session they bound and the four numbers
/// the queue's `Clock:` line prints, with the slowest frame named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ceilings {
    /// The session: the game (`client --lobby`, our own start), the frame
    /// it is drawn from and the frames drawn off-screen, playing.
    pub game: &'static str,
    pub from_frame: u32,
    pub frames: u32,
    /// Launch to frame zero, milliseconds.
    pub frame_zero_ms: u32,
    /// The drawn frames' 99th percentile and slowest, microseconds.
    pub p99_frame_us: u32,
    pub slowest_frame_us: u32,
    /// The slowest frame, counted from 1, and its cause in words, as the
    /// line names it: the phases inside it say which (`Clock::slowest_cause`
    /// is the longest), and `broken_by` holds the frame, not the words.
    pub slowest_at: u32,
    pub slowest_cause: &'static str,
    /// The sampled resident-set peak, MiB.
    pub peak_rss_mib: u32,
}

impl Ceilings {
    /// Each ceiling `c` breaks, as a sentence; empty when the session drew
    /// its frames, is under every ceiling and its slowest frame is the one
    /// named.
    pub fn broken_by(&self, c: &Clock) -> Vec<String> {
        let mut out = Vec::new();
        if c.frames != self.frames as usize {
            out.push(format!("{} frames drawn, not {}", c.frames, self.frames));
        }
        let over = |what: &str, ours: f64, ceiling: f64, unit: &str| {
            (ours > ceiling).then(|| format!("{what} {ours:.2} {unit} over its ceiling {ceiling}"))
        };
        out.extend(over(
            "frame zero",
            c.to_frame_zero_ms,
            f64::from(self.frame_zero_ms),
            "ms",
        ));
        out.extend(over(
            "p99 frame",
            c.p99_frame_ms,
            f64::from(self.p99_frame_us) / 1000.0,
            "ms",
        ));
        out.extend(over(
            "slowest frame",
            c.max_frame_ms,
            f64::from(self.slowest_frame_us) / 1000.0,
            "ms",
        ));
        out.extend(over(
            "peak RSS",
            c.peak_rss_kib as f64 / 1024.0,
            f64::from(self.peak_rss_mib),
            "MiB",
        ));
        if c.slowest_frame != self.slowest_at as usize {
            out.push(format!(
                "the slowest frame is frame {} ({:?} the longest phase in it), not frame {} \
                 ({})",
                c.slowest_frame, c.slowest_cause, self.slowest_at, self.slowest_cause
            ));
        }
        out
    }
}

/// **The pinned ceilings** (item 4161): `client --lobby --headless
/// --frame 0 --play --frames 600`, Great Lakes from our own start drawn
/// off-screen at 1920×1080, playing a tick a frame.
///
/// **The readings** (`docs/RENDERER.md` §3.11): eight sessions on
/// 2026-11-12 with three other lanes gating beside them (load average 14
/// to 20): frame zero 635 to 1,220 ms, median 705; p99 4.4 to 11.8 ms,
/// median 5.2; slowest 87 to 219 ms, median 100, frame 1 in all eight;
/// peak 239 to 256 MiB sampled, median 242. Each ceiling stands over the
/// worst of the eight with room for a gate at eight threads, and under
/// twice it: frame zero 1.5 s, p99 20 ms, slowest 400 ms, 320 MiB. **The slowest frame's
/// cause is first sight**: frame 1 decodes every piece's model, textures
/// and sprite it shows (`scene`, 36 to 57 ms in six of eight, the longest
/// phase), builds the ground's meshes (`ground`, 14 to 27 ms) and uploads
/// both (`gpu` and `wait`, 36 to 144 ms), where frame 3 on is 4 ms.
///
/// Before 4161, `--lobby` loaded the install's tables twice and built the
/// pair's runner only to drop it: 1,474 ms to frame zero on the still,
/// 427 MiB. A ceiling falls when the client gets faster and rises only on
/// a named regression.
pub const CLOCK: Ceilings = Ceilings {
    game: "GreatLakes lobby",
    from_frame: 0,
    frames: 600,
    frame_zero_ms: 1500,
    p99_frame_us: 20_000,
    slowest_frame_us: 400_000,
    slowest_at: 1,
    slowest_cause: "first sight",
    peak_rss_mib: 320,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Three frames, the second the slowest, its `ground` phase most of it;
    /// a phase outside every frame is no frame's cause.
    const TRACE: &str = r#"[
{"name":"open","cat":"client","ph":"X","ts":10,"dur":400000,"pid":1,"tid":1},
{"name":"frame zero","cat":"client","ph":"i","s":"p","ts":400100,"pid":1,"tid":1},
{"name":"gpu","cat":"client","ph":"X","ts":400200,"dur":1000,"pid":1,"tid":1},
{"name":"frame","cat":"client","ph":"X","ts":400150,"dur":2000,"pid":1,"tid":1},
{"name":"first pixels","cat":"client","ph":"i","s":"p","ts":402200,"pid":1,"tid":1},
{"name":"step","cat":"client","ph":"X","ts":402300,"dur":500,"pid":1,"tid":1},
{"name":"ground","cat":"client","ph":"X","ts":402900,"dur":6000,"pid":1,"tid":1},
{"name":"frame","cat":"client","ph":"X","ts":402250,"dur":9000,"pid":1,"tid":1},
{"name":"frame","cat":"client","ph":"X","ts":411300,"dur":3000,"pid":1,"tid":1},
{"name":"rss_kib","cat":"client","ph":"C","ts":411400,"pid":1,"tid":1,"args":{"rss_kib":512000}},
{"name":"quit","cat":"client","ph":"i","s":"p","ts":414400,"pid":1,"tid":1}
]"#;

    #[test]
    fn the_slowest_frame_is_named_with_the_longest_phase_inside_it() {
        let c = Clock::read(TRACE).unwrap();
        assert_eq!(c.frames, 3);
        assert_eq!(c.to_frame_zero_ms, 400.1);
        assert_eq!(c.to_first_pixels_ms, 402.2);
        assert_eq!((c.p99_frame_ms, c.max_frame_ms), (9.0, 9.0));
        assert_eq!(c.slowest_frame, 2);
        assert_eq!(c.slowest_cause, Some(("ground".to_string(), 6.0)));
        assert_eq!(c.peak_rss_kib, 512_000);
        assert!(Clock::read("[\n]").is_none(), "no frame zero, no clock");
    }

    /// **Made to fail**: each ceiling a microsecond or a MiB under the
    /// reading breaks, and the ceilings at the reading hold.
    #[test]
    fn a_ceiling_under_the_reading_breaks() {
        let c = Clock::read(TRACE).unwrap();
        let at = Ceilings {
            game: "test",
            from_frame: 0,
            frames: 3,
            frame_zero_ms: 401,
            p99_frame_us: 9000,
            slowest_frame_us: 9000,
            slowest_at: 2,
            slowest_cause: "ground",
            peak_rss_mib: 500,
        };
        assert_eq!(at.broken_by(&c), Vec::<String>::new());
        for under in [
            Ceilings {
                frame_zero_ms: 400,
                ..at
            },
            Ceilings {
                p99_frame_us: 8999,
                ..at
            },
            Ceilings {
                slowest_frame_us: 8999,
                ..at
            },
            Ceilings {
                peak_rss_mib: 499,
                ..at
            },
            Ceilings {
                slowest_at: 1,
                ..at
            },
            Ceilings { frames: 4, ..at },
        ] {
            assert_eq!(under.broken_by(&c).len(), 1, "{under:?}");
        }
    }
}
