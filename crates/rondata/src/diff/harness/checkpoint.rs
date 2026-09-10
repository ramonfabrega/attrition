//! Owned in-memory replay checkpoints. No serialization or source binding.
use super::*;

/// A replay and its recording cursor, cloned together at a record boundary.
///
/// Cloning retains the whole simulation, correction inputs and report prefix;
/// it is neither a compact snapshot nor a portable replay file. Callers must
/// supply the same remaining capture records. Source identity is not checked.
#[derive(Clone)]
pub struct ReplaySession {
    replay: Replay,
    stream: Option<crate::input::Stream>,
    next_record: usize,
}

impl ReplaySession {
    /// `init` must already include any sibling/trace corrections required by
    /// the replay. All borrowed setup data is consumed during construction.
    pub fn new(
        loaded: &Loaded,
        init: &Initial<'_>,
        tuning: Tuning,
        stream: Option<crate::input::Stream>,
    ) -> Self {
        Self {
            replay: Replay::new(loaded, init, tuning),
            stream,
            next_record: 0,
        }
    }

    /// Begin a fresh diagnostic interval. Clones retain the accumulated audit.
    pub fn enable_correction_audit(&mut self) {
        self.replay.built.correction_audit = Some(CorrectionAudit::default());
    }

    /// Suppress exactly one future Gaia reseating call, retaining its clock payloads.
    /// Refuse ambiguous, past, non-Gaia or seed-gated-out correction keys.
    pub fn fork_without_gaia_reseat(&self, key: ReseatKey) -> std::io::Result<Self> {
        let built = self.built();
        let matches = built
            .frame_guys
            .iter()
            .filter(|(n, _)| *n == key.frame)
            .flat_map(|(_, units)| units)
            .filter(|u| u.who == key.who && u.o == key.o)
            .count();
        if built.gaia_reseat_skip.is_some()
            || key.frame < built.sim.frame
            || key.who < 8
            || built
                .frame_guys
                .iter()
                .filter(|(n, _)| *n == key.frame)
                .count()
                != 1
            || matches != 1
            || !built.frame_seeds.iter().any(|(n, _)| *n == key.frame)
        {
            return Err(std::io::Error::other(
                "reseat key must identify one future seed-enabled Gaia correction",
            ));
        }
        let mut fork = self.clone();
        fork.replay.built.gaia_reseat_skip = Some(ReseatSkip { key, hits: 0 });
        fork.replay.built.notes.push(format!(
            "LAB INTERVENTION: skip Gaia reseat {key:?}; clocks and seed corrections retained"
        ));
        Ok(fork)
    }

    pub fn next_record(&self) -> usize {
        self.next_record
    }

    pub fn built(&self) -> &Built {
        &self.replay.built
    }

    /// Lab intervention: clone the checkpoint and omit future figure-record
    /// corrections (clock installation AND Gaia reseating). RNG reseeding, input cursor and past state remain intact.
    /// Refuse a vacuous intervention; this is not an alternative fidelity mode.
    pub fn fork_without_future_figure_corrections(&self) -> std::io::Result<Self> {
        let frame = self.replay.built.sim.frame;
        let removed = self
            .replay
            .built
            .frame_guys
            .iter()
            .filter(|(n, _)| *n >= frame)
            .count();
        if removed == 0 {
            return Err(std::io::Error::other(
                "no future figure corrections to remove",
            ));
        }
        let mut fork = self.clone();
        fork.replay.built.frame_guys.retain(|(n, _)| *n < frame);
        fork.replay.built.notes.push(format!(
            "LAB INTERVENTION: removed {removed} figure correction records from sim frame {frame}; seed corrections retained"
        ));
        Ok(fork)
    }

    /// Suppress only future clock payloads, retaining each frame's Gaia reseat
    /// inputs. This preserves the existing reseating and RNG policy.
    pub fn fork_without_future_clocks(&self) -> std::io::Result<Self> {
        let frame = self.replay.built.sim.frame;
        let clocks: usize = self
            .replay
            .built
            .frame_guys
            .iter()
            .filter(|(n, _)| *n >= frame)
            .flat_map(|(_, units)| units)
            .map(|u| u.guys.iter().filter(|g| g.has_clock()).count())
            .sum();
        if clocks == 0 {
            return Err(std::io::Error::other("no future clock payloads to remove"));
        }
        let mut fork = self.clone();
        for (_, units) in fork
            .replay
            .built
            .frame_guys
            .iter_mut()
            .filter(|(n, _)| *n >= frame)
        {
            for unit in units {
                // Preserve animation-category predicates and all reseat fields;
                // only make clock installation ineligible through guy_of.
                for guy in &mut unit.guys {
                    guy.cur_time = None;
                }
            }
        }
        fork.replay.built.notes.push(format!("LAB INTERVENTION: removed {clocks} future clock payloads from sim frame {frame}; Gaia reseat records and seed corrections retained"));
        Ok(fork)
    }

    /// Append the next source record. Repeated frame labels and forward gaps
    /// retain the harness semantics; skipped indices and backwards ticks fail
    /// before mutating state. Clone this session between calls to checkpoint it.
    pub fn push(&mut self, index: usize, frame: &Frame) -> std::io::Result<&FrameResult> {
        if index != self.next_record || frame.n < self.replay.last {
            return Err(std::io::Error::other(
                "nonconsecutive replay index or backwards frame",
            ));
        }
        let next = index
            .checked_add(1)
            .ok_or_else(|| std::io::Error::other("record index overflow"))?;
        self.replay
            .step(frame, &mut self.stream.as_mut(), &mut |_, _, _| {});
        self.next_record = next;
        Ok(self.replay.report.frames.last().unwrap())
    }

    /// Return the complete report, including its checkpointed prefix.
    pub fn finish(self) -> Report {
        self.replay.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::testkit::{trace, with_sibling_initials};
    use crate::testenv::{dump, install};
    use std::time::Instant;

    #[test]
    fn run69_checkpoint_repeats_complete_suffix_and_report() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run69-greatlakes-3k.txt") else {
            return;
        };
        let Some(trace) = trace("rontrace-run69.log") else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let start = Instant::now();
        let mut session = with_sibling_initials(|siblings| {
            source
                .with_replay_initial(|init| {
                    let mut init = init;
                    borrow_from_siblings(&mut init, siblings);
                    borrow_pasture(&mut init, &trace);
                    ReplaySession::new(&loaded, &init, Tuning::RON, None)
                })
                .unwrap()
        });
        for i in 0..1899 {
            session.push(i, &source.frame_state(i).unwrap()).unwrap();
        }
        let prefix_time = start.elapsed();
        assert!(!session.built().frame_seeds.is_empty());
        assert!(!session.built().frame_guys.is_empty());
        let clone_start = Instant::now();
        let checkpoint = session.clone();
        let clone_time = clone_start.elapsed();
        // Profile components without calling shallow vector capacity a full heap size.
        for (name, elapsed) in [
            ("sim", {
                let t = Instant::now();
                std::hint::black_box(checkpoint.replay.built.sim.clone());
                t.elapsed()
            }),
            ("report", {
                let t = Instant::now();
                std::hint::black_box(checkpoint.replay.report.clone());
                t.elapsed()
            }),
            ("corrections", {
                let t = Instant::now();
                std::hint::black_box((
                    checkpoint.replay.built.frame_seeds.clone(),
                    checkpoint.replay.built.frame_guys.clone(),
                ));
                t.elapsed()
            }),
        ] {
            eprintln!("checkpoint component clone+drop {name}: {elapsed:?}");
        }
        let b = &checkpoint.replay.built;
        let figure_bytes = b.frame_guys.capacity()
            * std::mem::size_of::<(i64, Vec<crate::gamelog::FrameUnit>)>()
            + b.frame_guys
                .iter()
                .map(|(_, units)| {
                    units.capacity() * std::mem::size_of::<crate::gamelog::FrameUnit>()
                        + units
                            .iter()
                            .map(|u| u.guys.capacity() * std::mem::size_of::<crate::gamelog::Guy>())
                            .sum::<usize>()
                })
                .sum::<usize>();
        eprintln!(
            "checkpoint allocation subset: seed vectors {} bytes, figure vectors {} bytes; report {} records, sim inline {} bytes (excludes owned heaps)",
            b.frame_seeds.capacity() * std::mem::size_of::<(i64, u32)>(),
            figure_bytes,
            checkpoint.replay.report.frames.len(),
            std::mem::size_of_val(&b.sim)
        );
        assert!(
            checkpoint.fork_without_future_figure_corrections().is_err(),
            "this late window has no future figure records"
        );
        assert!(checkpoint.fork_without_future_clocks().is_err());
        let mut authored = checkpoint.clone();
        let at = authored.built().sim.frame;
        authored.replay.built.frame_guys.push((at, Vec::new()));
        let original = authored.clone();
        let mut fork = authored.fork_without_future_figure_corrections().unwrap();
        assert!(fork.built().frame_guys.iter().all(|(n, _)| *n < at));
        assert_eq!(fork.built().frame_seeds, original.built().frame_seeds);
        // Normalize only the two declared changes; everything else remains identical.
        fork.replay.built.frame_guys = original.built().frame_guys.clone();
        fork.replay.built.notes = original.built().notes.clone();
        assert_eq!(
            format!("{:?}", fork.built()),
            format!("{:?}", original.built())
        );
        assert_eq!(fork.finish(), original.finish());
        let frame = source.frame_state(1899).unwrap();
        assert!(session.push(1900, &frame).is_err());
        assert_eq!(session.next_record(), 1899);
        let backwards = Frame {
            n: -1,
            ..Frame::default()
        };
        assert!(session.push(1899, &backwards).is_err());
        for i in 1899..1902 {
            session.push(i, &source.frame_state(i).unwrap()).unwrap();
        }
        let expected_built = format!("{:?}", session.built());
        let expected = session.finish();
        for _ in 0..2 {
            let start = Instant::now();
            let mut restored = checkpoint.clone();
            for i in restored.next_record()..1902 {
                restored.push(i, &source.frame_state(i).unwrap()).unwrap();
            }
            assert_eq!(format!("{:?}", restored.built()), expected_built);
            assert_eq!(restored.finish(), expected);
            eprintln!(
                "checkpoint: prefix {prefix_time:?}, clone {clone_time:?}, clone+suffix+checks {:?}",
                start.elapsed()
            );
        }
        // The private checkpointed prefix itself has not advanced with either branch.
        assert_eq!(checkpoint.next_record(), 1899);
        // Preserve existing frame-label semantics independently of the index cursor.
        let mut duplicate = checkpoint.clone();
        duplicate.push(1899, &frame).unwrap();
        duplicate.push(1900, &frame).unwrap();
        assert_eq!(
            duplicate.replay.report.frames[1899],
            duplicate.replay.report.frames[1900]
        );
        source.validate().unwrap();
    }

    #[test]
    fn recorded_checkpoint_preserves_cursor_selection_and_existing_report() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run7-ancient-nubian-orders.txt") else {
            return;
        };
        let Some(rec_path) = dump("Playback - 2026.08.24 10'15'53 (Mon).rcx") else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(path);
        let log = Log::parse(&text);
        let rec = crate::recgame::read(&rec_path).unwrap();
        let frames = log.frame_states();
        let mut stream = crate::input::Stream::new(&rec);
        let expected = run_with(&loaded, &log, Tuning::RON, None, Some(&mut stream)).unwrap();
        assert_eq!(expected.applied.orders, 9);
        let mut session = ReplaySession::new(
            &loaded,
            &log.replay_initial().unwrap(),
            Tuning::RON,
            Some(crate::input::Stream::new(&rec)),
        );
        let split = frames.len() / 2;
        for (i, frame) in frames.iter().enumerate().take(split) {
            session.push(i, frame).unwrap();
        }
        let checkpoint = session.clone();
        for (i, frame) in frames.iter().enumerate().skip(split) {
            session.push(i, frame).unwrap();
        }
        assert_eq!(session.finish(), expected);
        let mut restored = checkpoint.clone();
        for (i, frame) in frames.iter().enumerate().skip(split) {
            restored.push(i, frame).unwrap();
        }
        assert_eq!(restored.finish(), expected);
        // Counterexample: restoring sim while resetting the input stream is not a checkpoint.
        let mut broken = checkpoint;
        broken.stream = Some(crate::input::Stream::new(&rec));
        for (i, frame) in frames.iter().enumerate().skip(split) {
            broken.push(i, frame).unwrap();
        }
        assert_ne!(broken.finish(), expected, "stream reset must be observable");
    }
    #[test]
    fn run69_intervention_effects_are_visible_beyond_player_comparisons() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run69-greatlakes-3k.txt") else {
            return;
        };
        let Some(trace) = trace("rontrace-run69.log") else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let initial = with_sibling_initials(|siblings| {
            source
                .with_replay_initial(|init| {
                    let mut init = init;
                    borrow_from_siblings(&mut init, siblings);
                    borrow_pasture(&mut init, &trace);
                    ReplaySession::new(&loaded, &init, Tuning::RON, None)
                })
                .unwrap()
        });
        for split in [0, 94] {
            let mut control = initial.clone();
            let mut observed = initial.clone();
            observed.enable_correction_audit();
            for i in 0..split {
                control.push(i, &source.frame_state(i).unwrap()).unwrap();
            }
            let mut variants = [
                control.fork_without_future_figure_corrections().unwrap(),
                control.fork_without_future_clocks().unwrap(),
            ];
            // Prove isolation against every Built field, not just the measured outputs.
            let mut normalized = variants[1].clone();
            for ((n, units), (_, original)) in normalized
                .replay
                .built
                .frame_guys
                .iter_mut()
                .zip(&control.replay.built.frame_guys)
            {
                if *n >= control.replay.built.sim.frame {
                    for (unit, before) in units.iter_mut().zip(original) {
                        for (guy, old) in unit.guys.iter_mut().zip(&before.guys) {
                            assert!(crate::diff::setup::guy_of(guy).is_none());
                            guy.cur_time = old.cur_time;
                        }
                    }
                }
            }
            normalized.replay.built.notes = control.replay.built.notes.clone();
            assert_eq!(
                format!("{:?}", normalized.built()),
                format!("{:?}", control.built())
            );
            assert_eq!(normalized.next_record(), control.next_record());
            assert_eq!(normalized.finish(), control.clone().finish());
            let mut seen = [[None; 6]; 2];
            let mut affected = [(0usize, None); 2];
            for i in split..3000 {
                let frame = source.frame_state(i).unwrap();
                let expected = control.push(i, &frame).unwrap().clone();
                if split == 0 {
                    assert_eq!(observed.push(i, &frame).unwrap(), &expected);
                }
                for (which, branch) in variants.iter_mut().enumerate() {
                    let actual = branch.push(i, &frame).unwrap().clone();
                    let pairs: Vec<_> = control
                        .built()
                        .sim
                        .units
                        .iter()
                        .filter_map(|u| {
                            branch
                                .built()
                                .sim
                                .units
                                .iter()
                                .find(|v| v.owner == u.owner && v.index == u.index)
                                .map(|v| (u, v))
                        })
                        .collect();
                    let any_unit = pairs.iter().find(|(u, v)| u != v);
                    let clock = pairs.iter().find(|(u, v)| u.guys != v.guys);
                    let position = pairs.iter().find(|(u, v)| u.pos != v.pos);
                    let player = pairs
                        .iter()
                        .find(|(u, v)| usize::from(u.owner) < control.replay.players && u != v);
                    let flags = [
                        control.built().sim.units != branch.built().sim.units,
                        clock.is_some(),
                        position.is_some(),
                        player.is_some(),
                        control.built().sim.rng.seed != branch.built().sim.rng.seed,
                        expected != actual,
                    ];
                    if flags[0] {
                        affected[which].0 += 1;
                        affected[which].1 = Some(frame.n);
                    }
                    for (kind, differs) in flags.into_iter().enumerate() {
                        if differs && seen[which][kind].is_none() {
                            seen[which][kind] = Some(frame.n);
                            eprintln!(
                                "split {split}, policy {which} first {} at frame {}",
                                [
                                    "any unit",
                                    "clock",
                                    "position",
                                    "player unit",
                                    "RNG",
                                    "comparator"
                                ][kind],
                                frame.n
                            );
                            if let Some((u, v)) = match kind {
                                0 => any_unit,
                                1 => clock,
                                2 => position,
                                3 => player,
                                _ => None,
                            } {
                                eprintln!(
                                    "  unit {}/{}: position {:?} -> {:?}; clocks {:?} -> {:?}",
                                    u.owner, u.index, u.pos, v.pos, u.guys, v.guys
                                );
                            }
                        }
                    }
                }
            }
            eprintln!(
                "split {split}, policy effects [any unit, clock, position, player unit, RNG, comparator]: {seen:?}; affected counts/last: {affected:?}"
            );
            assert_eq!(seen, [[Some(95), None, None, None, None, None], [None; 6]]);
            assert_eq!(affected, [(11, Some(105)), (0, None)]);
            if split == 0 {
                let audit = observed.replay.built.correction_audit.take().unwrap();
                eprintln!("correction audit: {audit:#?}");
                assert!(audit.clock.attempted > 0);
                assert_eq!(audit.clock.changed, 0);
                assert!(audit.gaia_reseat.changed > 0);
                assert_eq!(
                    format!("{:?}", observed.built()),
                    format!("{:?}", control.built())
                );
                assert_eq!(observed.finish(), control.finish());
            }
        }
        source.validate().unwrap();
    }
    #[test]
    fn run69_single_reseat_interventions_follow_the_remaining_capture() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run69-greatlakes-3k.txt") else {
            return;
        };
        let Some(trace) = trace("rontrace-run69.log") else {
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let mut source = crate::capture::indexed::IndexedCapture::open(&path).unwrap();
        let initial = with_sibling_initials(|siblings| {
            source
                .with_replay_initial(|init| {
                    let mut init = init;
                    borrow_from_siblings(&mut init, siblings);
                    borrow_pasture(&mut init, &trace);
                    ReplaySession::new(&loaded, &init, Tuning::RON, None)
                })
                .unwrap()
        });
        let count = source.frames().len();
        assert_eq!(count, 3001);
        eprintln!(
            "RESEAT_RANGE records={count}, last_source_frame={}",
            source.frames()[count - 1].number
        );
        let mut scan = initial.clone();
        scan.enable_correction_audit();
        for i in 0..count {
            scan.push(i, &source.frame_state(i).unwrap()).unwrap();
        }
        let audit = scan.built().correction_audit.as_ref().unwrap();
        assert_eq!(
            audit.omitted_reseats, 0,
            "cannot call a truncated event list complete"
        );
        let keys = audit.changed_reseats.clone();
        assert_eq!(
            keys,
            (94..=100)
                .map(|frame| ReseatKey {
                    frame,
                    who: 8,
                    o: 0
                })
                .collect::<Vec<_>>()
        );
        drop(scan);
        let mut checkpoint = initial;
        let earliest = keys.iter().map(|k| k.frame).min().unwrap();
        while checkpoint.built().sim.frame < earliest {
            let i = checkpoint.next_record();
            checkpoint.push(i, &source.frame_state(i).unwrap()).unwrap();
        }
        assert_eq!(checkpoint.built().sim.frame, earliest);
        let key = keys[0];
        let mut ambiguous = checkpoint.clone();
        let record = ambiguous
            .built()
            .frame_guys
            .iter()
            .find(|(n, _)| *n == key.frame)
            .unwrap()
            .clone();
        ambiguous.replay.built.frame_guys.push(record);
        assert!(ambiguous.fork_without_gaia_reseat(key).is_err());
        let mut gated = checkpoint.clone();
        gated
            .replay
            .built
            .frame_seeds
            .retain(|(n, _)| *n != key.frame);
        assert!(gated.fork_without_gaia_reseat(key).is_err());
        drop((ambiguous, gated));
        for key in keys {
            let mut control = checkpoint.clone();
            let mut branch = checkpoint.fork_without_gaia_reseat(key).unwrap();
            // Only the explicit skip selector and provenance note differ at fork.
            let mut normalized = branch.clone();
            normalized.replay.built.gaia_reseat_skip = None;
            normalized.replay.built.notes = control.built().notes.clone();
            assert_eq!(
                format!("{:?}", normalized.built()),
                format!("{:?}", control.built())
            );
            assert_eq!(normalized.finish(), control.clone().finish());
            assert!(branch.fork_without_gaia_reseat(key).is_err());
            assert!(
                checkpoint
                    .fork_without_gaia_reseat(ReseatKey { who: 0, ..key })
                    .is_err()
            );
            assert!(
                checkpoint
                    .fork_without_gaia_reseat(ReseatKey { frame: -1, ..key })
                    .is_err()
            );
            let mut changed = 0;
            let mut first = None;
            let mut last = None;
            let mut effects = [None; 5];
            for i in checkpoint.next_record()..count {
                let frame = source.frame_state(i).unwrap();
                let expected = control.push(i, &frame).unwrap().clone();
                let actual = branch.push(i, &frame).unwrap().clone();
                let a = &control.built().sim;
                let b = &branch.built().sim;
                if a.units != b.units {
                    changed += 1;
                    last = Some(frame.n);
                    if first.is_none() {
                        first = Some(frame.n);
                        let id = a.unit_by_o(key.who as sim::Player, key.o as i16).unwrap();
                        let other = b.unit_by_o(key.who as sim::Player, key.o as i16).unwrap();
                        eprintln!(
                            "{key:?} first difference at source {}: path {:?} -> {:?}; orders {:?} -> {:?}",
                            frame.n,
                            a.units[id].path,
                            b.units[other].path,
                            a.units[id].orders,
                            b.units[other].orders
                        );
                    }
                }
                if key.frame == 99 && (102..=655).contains(&frame.n) {
                    assert_eq!(a.units.len(), b.units.len());
                    for (a, b) in a.units.iter().zip(&b.units).filter(|(a, b)| a != b) {
                        assert_eq!((a.owner, a.index), (8, 0));
                        if frame.n == 106 {
                            let original =
                                frame.units.iter().find(|u| u.who == 8 && u.o == 0).unwrap();
                            eprintln!(
                                "RESEAT_ORACLE source_frame=106 unit=8/0 original_angle={:?} control={} treatment={}",
                                original.angle, a.movement.heading.0, b.movement.heading.0
                            );
                            assert_eq!(original.angle, Some(i64::from(a.movement.heading.0)));
                            assert_ne!(original.angle, Some(i64::from(b.movement.heading.0)));
                        }
                        let mut normalized = b.clone();
                        normalized.movement.facing = a.movement.facing;
                        normalized.movement.frame_facing = a.movement.frame_facing;
                        normalized.movement.heading = a.movement.heading;
                        normalized.movement.des_angle = a.movement.des_angle;
                        assert_eq!(
                            &normalized, a,
                            "persistent difference beyond the four angles"
                        );
                    }
                }
                // Include membership in each projection, not just matched pairs.
                fn clocks(sim: &sim::Sim) -> Vec<(sim::Player, i16, &Vec<sim::anim::Guy>)> {
                    sim.units
                        .iter()
                        .map(|u| (u.owner, u.index, &u.guys))
                        .collect()
                }
                let positions = |sim: &sim::Sim| {
                    sim.units
                        .iter()
                        .map(|u| (u.owner, u.index, u.pos))
                        .collect::<Vec<_>>()
                };
                fn players(sim: &sim::Sim) -> Vec<&sim::Unit> {
                    sim.units
                        .iter()
                        .filter(|u| usize::from(u.owner) < sim.players.len())
                        .collect()
                }
                let flags = [
                    clocks(a) != clocks(b),
                    positions(a) != positions(b),
                    players(a) != players(b),
                    a.rng.seed != b.rng.seed,
                    expected != actual,
                ];
                for (n, flag) in flags.into_iter().enumerate() {
                    if flag && effects[n].is_none() {
                        effects[n] = Some(frame.n);
                    }
                }
            }
            assert_eq!(branch.built().gaia_reseat_skip.as_ref().unwrap().hits, 1);
            eprintln!(
                "RESEAT_RESULT {key:?}: changed_frames={changed}, first={first:?}, last={last:?}, first [clock,position,player,RNG,comparator]={effects:?}"
            );
            let expected_frames = if key.frame == 99 { 556 } else { 1 };
            let expected_last = if key.frame == 99 { 655 } else { key.frame + 1 };
            assert_eq!(
                (changed, first, last),
                (expected_frames, Some(key.frame + 1), Some(expected_last))
            );
            assert_eq!(effects, [None; 5]);
            let mut normalized = branch.built().clone();
            normalized.gaia_reseat_skip = None;
            normalized
                .notes
                .retain(|n| !n.starts_with("LAB INTERVENTION: skip Gaia reseat "));
            eprintln!(
                "RESEAT_FINAL {key:?}: complete_built_equal={}",
                format!("{normalized:?}") == format!("{:?}", control.built())
            );
            assert_eq!(format!("{normalized:?}"), format!("{:?}", control.built()));
            branch
                .replay
                .built
                .notes
                .retain(|n| !n.starts_with("LAB INTERVENTION: skip Gaia reseat "));
            assert_eq!(branch.finish(), control.finish());
        }
        source.validate().unwrap();
    }
}
