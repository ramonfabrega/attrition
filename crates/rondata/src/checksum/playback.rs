//! Score a window recording against a complete original playback.
mod diagnostic;
use crate::checksum::state;
use crate::diff::{built_from_lobby, from_lobby, sample::header_lobby};
use crate::trace::checksums::{complete_sets, per_frame};
use crate::trace::ranges::Record;
use std::path::Path;

pub struct Score {
    /// Setup plus every tick, including the terminal state.
    pub states: u32,
    pub agreed: [u32; 12],
    /// First differing state, our Adler (None for an unimplemented record)
    /// and the Adler in the original's trace, for each state line.
    pub first: [Option<(i32, Option<u32>, u32)>; 12],
    /// The first state our walk holds `Sim::game_over` on, if any.
    pub over: Option<i32>,
    /// True when the final scored state is the file's explicit closing package.
    pub through_closing: bool,
    /// Range evidence or capture instructions for each line's first mismatch.
    pub details: [Option<String>; 12],
}

/// **One take for every line a score parts on** (DECISIONS 148 §4): the
/// capture arguments that keep those lines' ranges and no others, to the
/// recording's requested end. A whole take, all twelve lines to the end,
/// reads about 9 GB off the disk while it runs and keeps 2 to 5; a row
/// that parts on one line asks for that line. `None` when no line parts.
/// The mask's bit is the line's index, as the single-line hints' is.
pub fn joined_take(
    first: &[Option<(i32, Option<u32>, u32)>; 12],
    requested: i32,
    early: bool,
) -> Option<String> {
    let mask = first
        .iter()
        .enumerate()
        .filter(|(_, f)| f.is_some())
        .fold(0u32, |m, (i, _)| m | 1 << i);
    (mask != 0).then(|| {
        format!(
            "--end-frame {requested}{} --tracer-def RON_CHECKSUMS --tracer-def RON_RANGES \
             --tracer-def RNG_MASK={mask:#x}u",
            if early { " --allow-early-end" } else { "" }
        )
    })
}

pub(crate) fn recording_start(
    loaded: &crate::load::Loaded,
    rec: &crate::recgame::RecGame,
) -> crate::diff::Built {
    let (info, players) = header_lobby(rec);
    let lobby = crate::lobby::Lobby { info, players };
    let (info, players) = lobby.borrowed();
    let start = from_lobby(loaded, &info, &players, sim::Tuning::RON);
    built_from_lobby(loaded, start).0
}

pub fn compare(install: &Path, recording: &Path, trace: &Path) -> Result<Score, String> {
    compare_until(install, recording, trace, None)
}

/// [`compare`], closing on state `until` where the original's game ended
/// before the recording's closing package (a conquest: the take's last set
/// is the game's last frame, item 4396), else on the closing package.
pub fn compare_until(
    install: &Path,
    recording: &Path,
    trace: &Path,
    until: Option<i32>,
) -> Result<Score, String> {
    score(install, recording, trace, Endpoint::Recording(until), None)
}

enum Endpoint {
    Recording(Option<i32>),
    Capture(i32),
}

/// Score every state of a verified native capture, including an intentionally short
/// capture of a longer recording. The caller verifies the receipt and recording identity.
pub fn diagnose_capture(
    install: &Path,
    recording: &Path,
    trace: &Path,
    last_state: i32,
    ranges: Option<&Path>,
) -> Result<Score, String> {
    let ranges = ranges
        .map(Record::open)
        .transpose()
        .map_err(|e| e.to_string())?;
    score(
        install,
        recording,
        trace,
        Endpoint::Capture(last_state),
        ranges,
    )
}

fn score(
    install: &Path,
    recording: &Path,
    trace: &Path,
    end: Endpoint,
    mut ranges: Option<Record>,
) -> Result<Score, String> {
    let rec = crate::recgame::read(&recording.to_string_lossy()).map_err(|e| e.to_string())?;
    let closing = rec
        .packages
        .last()
        .filter(|p| p.data.is_empty() && p.frame > 0)
        .map(|p| p.frame);
    let last = match end {
        Endpoint::Recording(limit) => {
            let closing = closing.ok_or("missing the recording's closing package")?;
            limit.map_or(closing, |u| u.min(closing))
        }
        Endpoint::Capture(last) => last,
    };
    if last < 0 {
        return Err("negative final state".into());
    }
    if !rec.truncated.is_empty() {
        return Err("truncated recording".into());
    }
    for p in &rec.packages {
        crate::commands::decode(&p.data).map_err(|e| format!("frame {}: {e:?}", p.frame))?;
    }
    let bytes = std::fs::read(trace).map_err(|e| format!("{}: {e}", trace.display()))?;
    let observed_end = match end {
        Endpoint::Recording(_) => None,
        Endpoint::Capture(last) => Some(last),
    };
    let trace_sets = complete_sets(&bytes, observed_end)?;
    let first = per_frame(&trace_sets);
    if let Endpoint::Capture(_) = end {
        if first.len() != last as usize + 1
            || first.first_key_value().map(|(&f, _)| f) != Some(-1)
            || first.last_key_value().map(|(&f, _)| f) != Some(i64::from(last) - 1)
        {
            return Err("checksum states do not span the capture receipt".into());
        }
        if let Some(record) = ranges.as_ref() {
            diagnostic::verify_record(record, &trace_sets, &bytes)?;
        }
    }
    let inst = crate::Install::new(install);
    let loaded = crate::load::load(&inst).map_err(|e| e.to_string())?;
    let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
    let rules = inst.rules().map_err(|e| e.to_string())?;
    let tribes = inst.tribe_defs(&rules).map_err(|e| e.to_string())?;
    let mut built = recording_start(&loaded, &rec);
    let mut stream = crate::input::Stream::pump(&rec);
    let mut agreed = [0u32; 12];
    let mut mismatch = [None; 12];
    let mut over = None;
    let mut details = [const { None }; 12];
    // Include the terminal state produced by the last tick. The original
    // tags a state's first set with the frame that just ran (setup is -1).
    for frame in 0..=last {
        let set = first
            .get(&(i64::from(frame) - 1))
            .ok_or_else(|| format!("missing checksum set for state {frame}"))?;
        if over.is_none() && built.sim.game_over() {
            over = Some(frame);
        }
        let lines = state::ranges(&built.sim, &look, &tribes);
        for (i, line) in lines.iter().enumerate() {
            let value = line.as_ref().map(|l| l.adler());
            if value == Some(set.lines[i]) {
                agreed[i] += 1;
            } else if mismatch[i].is_none() {
                mismatch[i] = Some((frame, value, set.lines[i]));
                if matches!(end, Endpoint::Capture(_)) {
                    details[i] = Some(diagnostic::explain(i, line.as_ref(), set, ranges.as_mut())?);
                }
            }
        }
        if frame < last {
            stream.apply(frame, &mut built);
            built.sim.command().tick();
        }
    }
    Ok(Score {
        states: last as u32 + 1,
        agreed,
        first: mismatch,
        over,
        details,
        through_closing: closing == Some(last),
    })
}

#[cfg(test)]
mod combat;

#[cfg(test)]
mod tests {
    use super::*;

    /// A take's mask is the parted lines' bits and nothing else: the
    /// second battery's row 130 parts units and leaders, lines 0 and 7,
    /// and the original ended it early (the pass of 2026-11-21).
    #[test]
    fn a_joined_take_keeps_the_parted_lines_and_no_others() {
        let mut first = [None; 12];
        assert_eq!(joined_take(&first, 6000, false), None);
        first[0] = Some((2354, Some(1), 2));
        first[7] = Some((4573, None, 3));
        assert_eq!(
            joined_take(&first, 6000, true).as_deref(),
            Some(
                "--end-frame 6000 --allow-early-end --tracer-def RON_CHECKSUMS \
                 --tracer-def RON_RANGES --tracer-def RNG_MASK=0x81u"
            )
        );
        first[11] = Some((1662, Some(1), 2));
        assert_eq!(
            joined_take(&first, 4500, false).as_deref(),
            Some(
                "--end-frame 4500 --tracer-def RON_CHECKSUMS --tracer-def RON_RANGES \
                 --tracer-def RNG_MASK=0x881u"
            )
        );
    }

    fn window_score(name: &str) -> Option<Score> {
        let install = crate::testenv::install_root()?;
        let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("ron-data/lab-experiments/2026-11-17-codex-live-recording");
        let game = crate::testenv::experiment(&dir, &format!("window-{name}.rcx"));
        let trace = crate::testenv::experiment(&dir, &format!("oracle-{name}/map-14/rontrace.log"));
        let (Some(game), Some(trace)) = (game, trace) else {
            eprintln!("skipping: {name} window recording or checksum take missing");
            return None;
        };
        Some(compare(install.as_ref(), &game, &trace).unwrap())
    }

    #[test]
    fn the_short_window_game_agrees_through_its_closing_state() {
        let Some(s) = window_score("short") else {
            return;
        };
        assert_eq!(s.states, 428);
        assert_eq!(s.first, [None; 12]);
        assert_eq!(s.agreed, [428; 12]);
    }

    #[test]
    fn the_long_window_game_matches_the_unit_and_group_ranges_at_the_army_split() {
        let Some(root) = crate::testenv::install_root() else {
            return;
        };
        let lab = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("ron-data/lab-experiments");
        let game = crate::testenv::experiment(
            &lab.join("2026-11-17-codex-live-recording"),
            "window-long.rcx",
        );
        let ranges = crate::testenv::experiment(
            &lab.join("2026-11-17-codex-window-9211"),
            "map-14/rontrace.ranges",
        );
        let (Some(game), Some(ranges)) = (game, ranges) else {
            return;
        };
        let mut original = crate::trace::ranges::Record::open(&ranges).unwrap();
        assert!(!original.truncated);
        let rec = crate::recgame::read(&game.to_string_lossy()).unwrap();
        let loaded = crate::load::load(&crate::Install::new(root)).unwrap();
        let mut built = recording_start(&loaded, &rec);
        let mut commands = crate::input::Stream::pump(&rec);
        for frame in 9210..=9475 {
            while built.sim.frame < frame {
                commands.apply(built.sim.frame as i32, &mut built);
                built.sim.command().tick();
            }
            let set = original.first_of(frame - 1).unwrap().clone();
            assert_eq!(set.flags, 0);
            let ranges = original.ranges(&set).unwrap();
            for (index, ours) in [
                (0, built.sim.checksum_units()),
                (5, built.sim.checksum_groups()),
            ] {
                let theirs: Vec<_> = ranges.iter().filter(|r| r.line == index).collect();
                assert_eq!(
                    ours.ranges.len(),
                    theirs.len(),
                    "state {frame} {}",
                    ours.name
                );
                for (a, b) in ours.ranges.iter().zip(theirs) {
                    assert_eq!(
                        a.bytes, b.bytes,
                        "state {frame} {} {} {}",
                        ours.name, a.record, a.field
                    );
                }
            }
        }
    }

    #[test]
    fn the_long_window_game_agrees_through_its_closing_state() {
        let Some(s) = window_score("long") else {
            return;
        };
        assert_eq!(s.states, 9476);
        assert_eq!(s.first, [None; 12]);
        assert_eq!(s.agreed, [9476; 12]);
    }
}
