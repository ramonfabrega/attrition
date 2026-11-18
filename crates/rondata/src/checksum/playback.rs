//! Score a window recording against a complete original playback.
use crate::checksum::state;
use crate::diff::{built_from_lobby, from_lobby, sample::header_lobby};
use crate::trace::checksums::{per_frame, sets, summary};
use std::path::Path;

pub struct Score {
    /// Setup plus every tick, including the terminal state.
    pub states: u32,
    pub agreed: [u32; 12],
    /// First differing state, our Adler (None for an unimplemented record)
    /// and the Adler in the original's trace, for each state line.
    pub first: [Option<(i32, Option<u32>, u32)>; 12],
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
    let rec = crate::recgame::read(&recording.to_string_lossy()).map_err(|e| e.to_string())?;
    let last = rec
        .packages
        .last()
        .filter(|p| p.data.is_empty() && p.frame > 0)
        .ok_or("missing the recording's closing package")?
        .frame;
    if !rec.truncated.is_empty() {
        return Err("truncated recording".into());
    }
    for p in &rec.packages {
        crate::commands::decode(&p.data).map_err(|e| format!("frame {}: {e:?}", p.frame))?;
    }
    let bytes = std::fs::read(trace).map_err(|e| format!("{}: {e}", trace.display()))?;
    let trace_sets = sets(&bytes);
    if summary(&bytes) != Some((trace_sets.len() as u32, 0, true)) {
        return Err("incomplete checksum take".into());
    }
    let first = per_frame(&trace_sets);
    let inst = crate::Install::new(install);
    let loaded = crate::load::load(&inst).map_err(|e| e.to_string())?;
    let look = crate::artdata::BuildLook::new(&inst).with_types(&loaded.build_types);
    let rules = inst.rules().map_err(|e| e.to_string())?;
    let tribes = inst.tribe_defs(&rules).map_err(|e| e.to_string())?;
    let mut built = recording_start(&loaded, &rec);
    let mut stream = crate::input::Stream::pump(&rec);
    let mut agreed = [0u32; 12];
    let mut mismatch = [None; 12];
    // Include the terminal state produced by the last tick. The original
    // tags a state's first set with the frame that just ran (setup is -1).
    for frame in 0..=last {
        let set = first
            .get(&(i64::from(frame) - 1))
            .ok_or_else(|| format!("missing checksum set for state {frame}"))?;
        let ours = state::adlers(&built.sim, &look, &tribes);
        for (i, value) in ours.into_iter().enumerate() {
            if value == Some(set.lines[i]) {
                agreed[i] += 1;
            } else {
                mismatch[i].get_or_insert((frame, value, set.lines[i]));
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn the_long_window_game_matches_both_deaths_until_the_pool_empties() {
        let Some(s) = window_score("long") else {
            return;
        };
        let mut first = [None; 12];
        first[0] = Some((9211, Some(1168710981), 874009299)); // units
        first[5] = Some((9211, Some(545805217), 3936441164)); // groups
        assert_eq!(s.states, 9476);
        assert_eq!(s.first, first);
        assert_eq!(
            s.agreed,
            [
                9211, 9476, 9476, 9476, 9476, 9470, 9476, 9476, 9476, 9476, 9476, 9476
            ]
        );
    }
}
