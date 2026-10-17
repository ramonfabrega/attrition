//! Bounded readers for tests that formerly retained entire captures.
use crate::capture::indexed::IndexedCapture;
use crate::gamelog::{Frame, FrameGuys, FrameUnit, Initial, Log};

fn verify() -> bool {
    std::env::var_os("RON_VERIFY_WHOLE_CAPTURE").is_some()
}
pub(crate) fn legacy() -> bool {
    std::env::var_os("RON_DISABLE_WHOLE_CAPTURE").is_some()
}

/// Visit adjacent positions, including duplicate/nonmonotonic labels. Never
/// join two files or skip a frame merely because it has no matching units.
pub(crate) fn frames(path: &str) -> Box<dyn Iterator<Item = Frame>> {
    if legacy() {
        let text = crate::capture::read(path);
        return Box::new(Log::parse(&text).frame_states().into_iter());
    }
    let mut source = IndexedCapture::open(path).unwrap();
    let mut expected = verify().then(|| {
        let text = crate::capture::read(path);
        let frames = Log::parse(&text).frame_states();
        assert_eq!(source.frames().len(), frames.len());
        frames.into_iter()
    });
    let mut at = 0;
    Box::new(std::iter::from_fn(move || {
        if at == source.frames().len() {
            return None;
        }
        let next = source.frame_state(at).unwrap();
        if let Some(expected) = &mut expected {
            assert_eq!(Some(&next), expected.next().as_ref(), "whole frame {at}");
        }
        at += 1;
        Some(next)
    }))
}

pub(crate) fn pairs(path: &str, mut visit: impl FnMut(&[Frame; 2])) {
    let mut previous = None;
    for next in frames(path) {
        if let Some(before) = previous.take() {
            let pair = [before, next];
            visit(&pair);
            let [_, after] = pair;
            previous = Some(after);
        } else {
            previous = Some(next);
        }
    }
}

pub(crate) fn with_initial<R>(
    source: &mut IndexedCapture,
    path: &str,
    use_initial: impl FnOnce(Initial<'_>) -> R,
) -> R {
    source
        .with_replay_initial(|init| {
            if verify() {
                let text = crate::capture::read(path);
                let log = Log::parse(&text);
                let mut expected = log.initial().unwrap();
                expected.frame_bodies.clear();
                assert_eq!(init, expected, "whole retained Initial: {path}");
            }
            use_initial(init)
        })
        .unwrap()
}

/// Use the exact position-body reader and filter from Log::initial, with its
/// frame-minus-one clock. Empty body rows do not create an audit frame.
fn bodies(source: &mut IndexedCapture, at: usize) -> Vec<FrameUnit> {
    let raw = source.read_frame(at).unwrap();
    let log = Log::parse(&raw);
    let frames = log.frames();
    assert_eq!(frames.len(), 1);
    crate::gamelog::observation_rows(frames[0].1, true)
        .into_iter()
        .filter(|u| u.guys.iter().any(|g| g.pos.is_some()))
        .collect()
}
pub(crate) struct Bodies {
    source: IndexedCapture,
    old: Option<FrameGuys>,
}
impl Bodies {
    pub(crate) fn open(path: &str) -> Self {
        let mut source = IndexedCapture::open(path).unwrap();
        let old = (legacy() || verify()).then(|| {
            let text = crate::capture::read(path);
            Log::parse(&text).initial().unwrap().frame_bodies
        });
        if verify() {
            let mut at_old = 0;
            for at in 0..source.frames().len() {
                let rows = bodies(&mut source, at);
                if rows.is_empty() {
                    continue;
                }
                assert_eq!(
                    (source.frames()[at].number - 1, &rows),
                    (
                        old.as_ref().unwrap()[at_old].0,
                        &old.as_ref().unwrap()[at_old].1
                    )
                );
                at_old += 1;
            }
            assert_eq!(at_old, old.as_ref().unwrap().len());
        }
        Self {
            source,
            old: if legacy() { old } else { None },
        }
    }
    pub(crate) fn last(&mut self) -> i64 {
        if let Some(old) = &self.old {
            return old.last().map_or(0, |(n, _)| *n);
        }
        for at in (0..self.source.frames().len()).rev() {
            if !bodies(&mut self.source, at).is_empty() {
                return self.source.frames()[at].number - 1;
            }
        }
        0
    }
    pub(crate) fn at(&mut self, n: i64) -> Option<Vec<FrameUnit>> {
        if let Some(old) = &self.old {
            return old
                .iter()
                .find(|(f, _)| *f == n)
                .map(|(_, rows)| rows.clone());
        }
        for at in 0..self.source.frames().len() {
            if self.source.frames()[at].number - 1 != n {
                continue;
            }
            let rows = bodies(&mut self.source, at);
            if !rows.is_empty() {
                return Some(rows);
            }
        }
        None
    }
}

pub(crate) fn build<'a>(
    loaded: &crate::load::Loaded,
    mut init: Initial<'a>,
    siblings: &[&Initial<'a>],
    trace: Option<&crate::trace::Trace>,
) -> crate::diff::Built {
    crate::diff::setup::borrow_from_siblings(&mut init, siblings);
    if let Some(trace) = trace {
        crate::diff::setup::borrow_pasture(&mut init, trace);
    }
    crate::diff::setup::build_sim(loaded, &init, sim::Tuning::RON)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "ron-whole-capture-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&p).unwrap();
        p
    }
    fn body(n: i64, x: i64) -> String {
        format!(
            " BEGIN FRAME {n}\n  BEGIN UNITDATA\n   BEGIN OBJECT\n    BEGIN SUBOBJECT\n     flags 1\n     who 0\n     o 7\n   BEGIN GUY\n    x {x}\n    y 2\n    z 3\n    angle 4\n"
        )
    }
    #[test]
    fn rolling_pairs_preserve_duplicate_gap_empty_and_file_boundaries() {
        let root = root();
        for (i, text) in [
            "BEGIN GAME\n".to_string(),
            format!("BEGIN GAME\n{}", body(7, 1)),
            format!(
                "BEGIN GAME\n{}{} BEGIN FRAME 1\n{}BEGIN FRAME 99\n",
                body(3, 4),
                body(3, 5),
                body(8, 6)
            ),
        ]
        .iter()
        .enumerate()
        {
            let path = root.join(format!("{i}.txt"));
            std::fs::write(&path, text.replace('\n', "\r\n")).unwrap();
            let expected = Log::parse(text).frame_states();
            let mut got = Vec::new();
            pairs(path.to_str().unwrap(), |p| got.push(p.clone()));
            assert_eq!(
                got,
                expected
                    .windows(2)
                    .map(|p| [p[0].clone(), p[1].clone()])
                    .collect::<Vec<_>>()
            );
            if i == 2 {
                assert_eq!(
                    got.iter().map(|p| (p[0].n, p[1].n)).collect::<Vec<_>>(),
                    vec![(3, 3), (3, 1), (1, 8)]
                );
            } else {
                assert!(got.is_empty());
            }
        }
        std::fs::remove_dir_all(root).unwrap(); // only this test's authored files
    }
    #[test]
    fn bounded_bodies_keep_frame_offset_first_match_and_last_nonempty_position() {
        let root = root();
        let path = root.join("bodies.txt");
        let text = format!(
            "BEGIN GAME\n{}{} BEGIN FRAME 1\n{}{} BEGIN FRAME 9\n",
            body(3, 7),
            body(3, 8),
            body(1, 9),
            body(5, 11)
        );
        std::fs::write(&path, &text).unwrap();
        let expected = Log::parse(&text).initial().unwrap().frame_bodies;
        assert_eq!(
            expected.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            vec![2, 2, 0, 4]
        );
        let mut source = Bodies::open(path.to_str().unwrap());
        assert_eq!(source.last(), 4);
        for n in -1..10 {
            assert_eq!(
                source.at(n),
                expected
                    .iter()
                    .find(|(f, _)| *f == n)
                    .map(|(_, rows)| rows.clone())
            );
        }
        assert_eq!(source.at(2).unwrap()[0].guys[0].pos.unwrap().x, 7);
        assert_eq!(source.at(0).unwrap()[0].guys[0].pos.unwrap().x, 9);
        drop(source);
        std::fs::remove_dir_all(root).unwrap(); // authored files only
    }
}
