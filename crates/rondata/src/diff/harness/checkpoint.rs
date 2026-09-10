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

    pub fn next_record(&self) -> usize {
        self.next_record
    }

    pub fn built(&self) -> &Built {
        &self.replay.built
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
}
