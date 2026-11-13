//! The checksum hook's records (item 4214, step H): the original's fifteen
//! `check_all` lines and their `total`, every frame, from a `rontrace.log`
//! written by a `RON_CHECKSUMS` build of `tools/trace/`
//! (`tools/trace/checksums.h` has the hook and the fold).
//!
//! `CheckSums::check_all@00936560` runs on every turn and prints only
//! inside a dump window. The hook reads each line at the entry of
//! `SyncLogger::logToMemory<unsigned_long_&>@00463930`, where `check_all`
//! hands it, and writes a set whenever its frame or any of its sixteen
//! words differs from the last set written. A set's `frame` is the
//! tracer's: `Game::frame` at the last `Game::do_frame` entry, `-1`
//! before the first — so every set tagged `f` was taken after frame `f`'s
//! tick and its `end_frame`, and its `ordinal` 0 is the first `check_all`
//! after it (`docs/CHECKSUM.md` §24).

/// The record kind the hook writes (`tracer.c`'s `K_CSUM`).
pub const K_CSUM: u32 = 13;

/// One of `check_all`'s sets as the hook wrote it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Set {
    /// The tracer's frame: `Game::frame` at the last `do_frame` entry.
    pub frame: i64,
    /// The set's place among the frame's `check_all` calls, 0 the first.
    pub ordinal: u32,
    /// `check_all`'s return address into `process_turn`, at the image
    /// base [`super::IMAGE_BASE`]: which of its two calls made the set.
    pub caller: u32,
    /// The frame's `check_all` calls up to and including this one.
    pub calls: u32,
    /// The fifteen lines, in [`crate::checksum::CHECKSUM_LINES`]' order.
    pub lines: [u32; 15],
    /// `check_all`'s wrapping sum of the fifteen.
    pub total: u32,
}

/// The hook's attach-to-detach count: `(sets written, torn sets, whether
/// the stub passed esp)` — `I_CSUM` (INFO 35), written at detach. `None`
/// for a log that never detached or no `RON_CHECKSUMS` build wrote.
pub fn summary(bytes: &[u8]) -> Option<(u32, u32, bool)> {
    records(bytes)
        .filter(|r| r[0] == 5 && r[1] == 35)
        .last()
        .map(|r| (r[2], r[3], r[4] == 1))
}

fn records(bytes: &[u8]) -> impl Iterator<Item = [u32; 8]> + '_ {
    let base_ok = bytes.len() >= 32;
    bytes
        .get(32..)
        .filter(|_| base_ok)
        .unwrap_or(&[])
        .as_chunks::<32>()
        .0
        .iter()
        .map(|c| {
            let mut r = [0u32; 8];
            for (i, w) in r.iter_mut().enumerate() {
                *w = u32::from_le_bytes(c[i * 4..i * 4 + 4].try_into().unwrap());
            }
            r
        })
}

/// Every set the log holds, in log order. A set is four records, parts 0
/// to 3, written back to back from one call; a part out of order drops
/// the set it would have joined, so a set is whole or absent.
pub fn sets(bytes: &[u8]) -> Vec<Set> {
    let base = if bytes.len() >= 32 {
        u32::from_le_bytes(bytes[8..12].try_into().unwrap())
    } else {
        super::IMAGE_BASE
    };
    let mut out = Vec::new();
    let mut parts: Vec<[u32; 8]> = Vec::new();
    for r in records(bytes).filter(|r| r[0] == K_CSUM) {
        let part = r[1] & 0xff;
        if part as usize != parts.len() || parts.first().is_some_and(|p| p[1] >> 8 != r[1] >> 8) {
            parts.clear();
            if part != 0 {
                continue;
            }
        }
        parts.push(r);
        if parts.len() < 4 {
            continue;
        }
        let w: Vec<u32> = [
            &parts[0][3..7],
            &parts[1][2..7],
            &parts[2][2..7],
            &parts[3][2..3],
        ]
        .concat();
        let caller = parts[0][2];
        out.push(Set {
            frame: i64::from(parts[0][7] as i32),
            ordinal: parts[0][1] >> 8,
            caller: if caller == 0 {
                0
            } else {
                caller.wrapping_sub(base).wrapping_add(super::IMAGE_BASE)
            },
            calls: parts[3][4],
            lines: w.try_into().unwrap(),
            total: parts[3][3],
        });
        parts.clear();
    }
    out
}

/// The first set of each frame, in frame order: the state at the end of
/// that frame's tick, before any package of the next turn.
pub fn per_frame(sets: &[Set]) -> std::collections::BTreeMap<i64, &Set> {
    let mut out = std::collections::BTreeMap::new();
    for s in sets {
        out.entry(s.frame).or_insert(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(kind: u32, a: u32, rest: [u32; 5], frame: i32) -> [u8; 32] {
        let words = [
            kind,
            a,
            rest[0],
            rest[1],
            rest[2],
            rest[3],
            rest[4],
            frame as u32,
        ];
        let mut out = [0u8; 32];
        for (i, w) in words.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        out
    }

    /// The four parts of one set, as `tracer.c`'s `on_hook` writes them.
    fn set(frame: i32, ordinal: u32, first: u32) -> Vec<[u8; 32]> {
        let v: Vec<u32> = (0..16).map(|i| first + i).collect();
        let o = ordinal << 8;
        vec![
            rec(K_CSUM, o, [0x0093_fb68, v[0], v[1], v[2], v[3]], frame),
            rec(K_CSUM, o | 1, [v[4], v[5], v[6], v[7], v[8]], frame),
            rec(K_CSUM, o | 2, [v[9], v[10], v[11], v[12], v[13]], frame),
            rec(K_CSUM, o | 3, [v[14], v[15], ordinal + 1, 0, 0], frame),
        ]
    }

    fn log(recs: impl IntoIterator<Item = [u8; 32]>) -> Vec<u8> {
        let mut out = rec(0x544E_4F52, 2, [0x0040_0000, 0x1000, 0, 0, 0], 0).to_vec();
        for r in recs {
            out.extend_from_slice(&r);
        }
        out
    }

    /// A set is read whole, in the line order the hook wrote it; a torn
    /// one is dropped; the per-frame fold keeps each frame's first.
    #[test]
    fn a_set_is_four_records_read_whole() {
        let mut recs = set(90, 0, 100);
        // A part 2 with no part 1 before it: the set it began is dropped.
        let torn = set(90, 1, 200);
        recs.extend([torn[0], torn[2], torn[3]]);
        recs.extend(set(90, 3, 300));
        recs.extend(set(91, 0, 400));
        recs.push(rec(5, 35, [3, 0, 1, 0, 0], 91));
        let bytes = log(recs);
        let got = sets(&bytes);
        assert_eq!(got.len(), 3, "{got:?}");
        assert_eq!(got[0].frame, 90);
        assert_eq!(got[0].lines[0], 100);
        assert_eq!(got[0].lines[14], 114);
        assert_eq!(got[0].total, 115);
        assert_eq!(got[0].caller, 0x0093_fb68);
        assert_eq!((got[1].ordinal, got[1].calls, got[1].lines[0]), (3, 4, 300));
        let first = per_frame(&got);
        assert_eq!(first[&90].lines[0], 100, "the frame's first set");
        assert_eq!(first[&91].lines[5], 405);
        assert_eq!(summary(&bytes), Some((3, 0, true)));
    }
}
