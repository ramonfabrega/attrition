//! Bounded diagnostic exports over the existing read-only replay observer.
mod fields;

use crate::{
    diff::{Built, FrameResult, Report},
    gamelog::Frame,
};
use std::{fmt::Write as _, fs::OpenOptions, io::Write, path::Path};

/// Paths and invocation supplied by the caller; no replay inputs are inferred.
#[derive(Clone, Default)]
pub struct Metadata {
    pub capture: String,
    pub source_bytes: usize,
    pub siblings: Vec<String>,
    pub trace: Option<String>,
    pub recording: Option<String>,
    pub reproduce: String,
    /// Export-relative initial selection for a failure artifact.
    pub focus_index: Option<usize>,
}

fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            c if c.is_control() => {
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn strings(xs: &[String]) -> String {
    format!(
        "[{}]",
        xs.iter().map(|s| quoted(s)).collect::<Vec<_>>().join(",")
    )
}
fn point(p: sim::Pos) -> String {
    format!("[{},{}]", p.x, p.y)
}
fn row(
    who: i64,
    o: i64,
    original: Option<&crate::gamelog::UnitDump>,
    ours: Option<&sim::Unit>,
    scope: bool,
) -> String {
    let a = original.map_or("null".into(), |u| format!("[{},{}]", u.pos.x, u.pos.y));
    let b = ours.map_or("null".into(), |u| point(u.pos));
    let original_path = original.map_or(Vec::new(), |u| {
        u.path
            .iter()
            .rev()
            .map(|p| format!("[{},{}]", p.to.0, p.to.1))
            .collect()
    });
    let rust_path = ours.map_or(Vec::new(), |u| {
        u.path.iter().rev().map(|p| point(p.to)).collect()
    });
    format!(
        "{{\"id\":{},\"who\":{who},\"o\":{o},\"original\":{a},\"rust\":{b},\"scope\":{scope},\"originalPath\":[{}],\"rustPath\":[{}],\"originalRecord\":{},\"rustRecord\":{}}}",
        quoted(&format!("{who}/{o}")),
        original_path.join(","),
        rust_path.join(","),
        quoted(&original.map_or("Not present in this frame".into(), |u| format!("{u:#?}"))),
        quoted(
            &ours.map_or("Not linked in this comparison".into(), |u| format!(
                "{u:#?}"
            ))
        )
    )
}

/// Stores only a selected window. Bounds apply to serialized data, not replay RSS.
#[derive(Default)]
pub struct Window {
    start: usize,
    count: usize,
    visited: usize,
    frames: Vec<String>,
    bytes: usize,
    too_large: bool,
    geometry: (i32, i32),
    seed_inputs: usize,
    guy_inputs: usize,
}
impl Window {
    pub fn new(start: usize, count: usize) -> Result<Self, &'static str> {
        Self::resume(start, count, 0)
    }

    /// Observe a continuation whose next record has this global source index.
    pub fn resume(start: usize, count: usize, next_record: usize) -> Result<Self, &'static str> {
        if next_record > start {
            return Err("continuation begins after the requested window");
        }
        if !(1..=200).contains(&count) {
            return Err("count must be 1..200");
        }
        Ok(Self {
            start,
            count,
            visited: next_record,
            ..Self::default()
        })
    }
    pub fn observe(&mut self, built: &Built, frame: &Frame, result: &FrameResult) {
        let index = self.visited;
        self.visited += 1;
        if index < self.start || index - self.start >= self.count || self.too_large {
            return;
        }
        self.geometry = (
            built.sim.world.width() * sim::world::UNITS_PER_CELL,
            built.sim.world.height() * sim::world::UNITS_PER_CELL,
        );
        self.seed_inputs = built.frame_seeds.len();
        self.guy_inputs = built.frame_guys.len();
        let players = built.sim.players.len() as i64;
        let mut units = Vec::new();
        for unit in &frame.units {
            let scope = (0..players).contains(&unit.who);
            let linked = if scope {
                built
                    .units
                    .iter()
                    .find(|l| l.who == unit.who && l.o == unit.o)
                    .map(|l| l.unit)
                    .or_else(|| {
                        i16::try_from(unit.o)
                            .ok()
                            .and_then(|o| built.sim.unit_by_o(unit.who as sim::Player, o))
                    })
            } else {
                None
            };
            units.push(row(
                unit.who,
                unit.o,
                Some(unit),
                linked.map(|id| &built.sim.units[id]),
                scope,
            ));
        }
        for &(who, o) in &result.extra_units {
            let ours = i16::try_from(o)
                .ok()
                .and_then(|o| built.sim.unit_by_o(who as sim::Player, o))
                .map(|id| &built.sim.units[id]);
            units.push(row(who, o, None, ours, true));
        }
        let issues = result.diverged.len()
            + result.unlinked
            + result.extra_units.len()
            + result.order_diverged.len()
            + result.los_diverged.len()
            + result.packed_diverged.len()
            + result.angle_diverged.len()
            + result.collide_diverged.len()
            + result.gather_diverged.len()
            + result.build_unlinked
            + result.build_diverged.len()
            + result.queue_diverged.len()
            + result.city_diverged.len()
            + result.city_unlinked;
        let differences = fields::differences(result)
            .iter()
            .map(fields::Difference::json)
            .collect::<Vec<_>>()
            .join(",");
        let json = format!(
            "{{\"n\":{},\"index\":{index},\"issues\":{issues},\"compared\":{},\"positionMismatches\":{},\"orderCompared\":{},\"rng\":{},\"seedInstalledThisTick\":{},\"differences\":[{differences}],\"units\":[{}],\"report\":{}}}",
            frame.n,
            result.compared,
            result.diverged.len(),
            result.order_compared,
            built.sim.rng.seed,
            built.frame_seeds.iter().any(|(n, _)| *n == frame.n - 1),
            units.join(","),
            quoted(&format!("{result:#?}"))
        );
        self.bytes += json.len();
        if self.bytes > 64 * 1024 * 1024 {
            self.too_large = true;
            return;
        }
        self.frames.push(json);
    }
    pub fn write(
        &self,
        report: &Report,
        meta: &Metadata,
        output: &Path,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if self.too_large {
            return Err("export exceeds 64 MiB; reduce --count".into());
        }
        if self.frames.is_empty() {
            return Err("empty export".into());
        }
        let revision = std::process::Command::new("git")
            .args(["describe", "--always", "--dirty"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map_or("unknown".into(), |o| {
                String::from_utf8_lossy(&o.stdout).trim().to_owned()
            });
        let seed_inputs = self.seed_inputs;
        let guy_inputs = self.guy_inputs;
        let focus = meta.focus_index.map_or("null".into(), |i| i.to_string());
        let data = format!(
            "{{\"schema\":1,\"focusIndex\":{focus},\"capture\":{},\"sourceBytes\":{},\"revision\":{},\"siblings\":{},\"trace\":{},\"recording\":{},\"seedInputs\":{seed_inputs},\"guyInputs\":{guy_inputs},\"world\":[{},{}],\"notes\":{},\"applied\":{},\"reproduce\":{},\"frames\":[{}]}}",
            quoted(&meta.capture),
            meta.source_bytes,
            quoted(&revision),
            strings(&meta.siblings),
            quoted(meta.trace.as_deref().unwrap_or("Not supplied")),
            quoted(meta.recording.as_deref().unwrap_or("Not supplied")),
            self.geometry.0,
            self.geometry.1,
            strings(&report.notes),
            quoted(&format!("{:#?}", report.applied)),
            quoted(&meta.reproduce),
            self.frames.join(",")
        );
        if data.len() > 64 * 1024 * 1024 {
            return Err("export exceeds 64 MiB; reduce --count".into());
        }
        let template = include_str!("../../../tools/viewer/viewer.html");
        let html = template
            .replace(
                "/*__VIEWER_CODE__*/",
                include_str!("../../../tools/viewer/viewer.js"),
            )
            .replace("__REPLAY_DATA__", &data);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?;
        file.write_all(html.as_bytes())?;
        Ok(html.len())
    }
}
/// Export around a test-selected failure using its own replay closure.
/// Passing `None` does no filesystem work and never invokes `replay`.
/// The complete second report must equal the first; a consumed order stream or
/// changed input therefore cannot silently produce evidence for another run.
pub fn write_failure(
    expected: &Report,
    failure: Option<(usize, &str)>,
    meta: &Metadata,
    output: &Path,
    replay: impl FnOnce(&mut Window) -> Option<Report>,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let Some((index, reason)) = failure else {
        return Ok(None);
    };
    if index >= expected.frames.len() {
        return Err("failure index outside report".into());
    }
    if output.exists() {
        return Err("output already exists; choose a new file".into());
    }
    let start = index.saturating_sub(8);
    let count = (index.saturating_add(9).min(expected.frames.len())) - start;
    let mut window = Window::new(start, count)?;
    let mut actual = replay(&mut window).ok_or("diagnostic replay lacks initial state")?;
    if actual != *expected {
        return Err("diagnostic replay changed the report; artifact refused".into());
    }
    actual.notes.push(format!(
        "Failure selected by test: source index {index}, frame {}: {reason}",
        expected.frames[index].frame
    ));
    let mut meta = meta.clone();
    meta.focus_index = Some(index - start);
    window.write(&actual, &meta, output).map(Some)
}

/// Unique default location for local failure artifacts, outside the workspace.
/// Call only after finding a failure; errors must not replace the test assertion.
pub fn failure_path(label: &str) -> std::io::Result<std::path::PathBuf> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join("attrition-diff-failures");
    std::fs::create_dir_all(&directory)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_nanos();
    let label: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    Ok(directory.join(format!(
        "{label}-{}-{stamp}-{}.html",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resumed_window_keeps_the_global_cursor_and_refuses_a_late_start() {
        assert!(Window::resume(10, 3, 11).is_err());
        assert!(Window::resume(10, 0, 10).is_err());
        let resumed = Window::resume(10, 3, 7).unwrap();
        assert_eq!(resumed.visited, 7);
        assert_eq!(resumed.start, 10);
        assert_eq!(Window::new(10, 3).unwrap().visited, 0);
    }

    #[test]
    fn passing_check_does_not_replay_or_write() {
        let result = write_failure(
            &Report::default(),
            None,
            &Metadata::default(),
            Path::new("/does-not-exist/no-artifact.html"),
            |_| panic!("passing test replayed"),
        );
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn changed_replay_is_refused_before_writing() {
        let expected = Report {
            frames: vec![FrameResult::default()],
            ..Default::default()
        };
        let output = failure_path("changed-report-test").unwrap();
        let error = write_failure(
            &expected,
            Some((0, "test")),
            &Metadata::default(),
            &output,
            |_| Some(Report::default()),
        )
        .unwrap_err();
        assert!(error.to_string().contains("changed the report"));
        assert!(!output.exists());
    }

    #[test]
    fn selected_failure_exports_actual_replay_context() {
        let Some(install) = crate::testenv::install() else {
            return;
        };
        let Some(path) = crate::testenv::dump("gamelog-run6-ancient-nubian-builds7.txt") else {
            return;
        };
        let loaded = crate::load::load(&install).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let log = crate::gamelog::Log::parse(&text);
        let expected =
            crate::diff::run_traced(&loaded, &log, sim::Tuning::RON, Some(20), None, &[], None)
                .unwrap();
        let output = failure_path("intentional-acceptance-test").unwrap();
        let meta = Metadata { capture: path, source_bytes: text.len(), reproduce: "cargo test -p rondata --release selected_failure_exports_actual_replay_context -- --nocapture".into(), ..Default::default() };
        let bytes = write_failure(
            &expected,
            Some((
                10,
                "intentional acceptance-test failure; replay records are unmodified",
            )),
            &meta,
            &output,
            |window| {
                crate::diff::run_traced_observed(
                    &loaded,
                    &log,
                    sim::Tuning::RON,
                    Some(20),
                    None,
                    &[],
                    None,
                    |b, f, r| window.observe(b, f, r),
                )
            },
        )
        .unwrap()
        .unwrap();
        let html = std::fs::read_to_string(&output).unwrap();
        assert_eq!(bytes, html.len());
        assert!(html.contains("Failure selected by test: source index 10"));
        assert!(html.contains("\"index\":2,"));
        assert!(html.contains("\"index\":18,"));
        assert!(!html.contains("\"index\":19,"));
        eprintln!("intentional acceptance-test artifact: {}", output.display());
        // create_new refusal happens before another replay or any overwrite.
        assert!(
            write_failure(&expected, Some((10, "again")), &meta, &output, |_| panic!(
                "existing output replayed"
            ))
            .is_err()
        );
        assert_eq!(std::fs::read_to_string(output).unwrap(), html);
    }

    #[test]
    fn json_string_cannot_close_the_embedded_data_script() {
        assert_eq!(
            quoted("</script>\n\"\\"),
            "\"\\u003c/script>\\u000a\\\"\\\\\""
        );
    }
}
