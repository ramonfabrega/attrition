//! The draw-site trace — reading `rontrace.log` from Rust.
//!
//! `tools/trace/` is the instrument (`docs/ORACLE.md`, "The draw-site trace
//! and function coverage"); `tools/trace/report.py` has been the only
//! reader of it since it existed. This is the second, and it exists for
//! one reason: **a count is a weak assertion and the trace carries a
//! sequence**. Every draw record names the *site* that took it — the
//! return address into the calling function — so a mechanic can be checked
//! draw for draw rather than by a total, and checked that way even while
//! the frame's stream has not yet been made to line up
//! (`docs/SCOUT.md` §10 is the first such check).
//!
//! Nothing here reads the user's install: a trace is a file the user's own
//! run wrote, taken by path exactly as a `gamelog.txt` is, and nothing
//! from it is committed.
//!
//! ## The format
//!
//! A 32-byte header then 32-byte records, each eight little-endian `u32`:
//! `[kind, a, b, c, d, e, f, frame]` (`tools/trace/tracer.c`'s `emit`). The
//! header is a record whose `kind` is the ASCII `RONT`, carrying the
//! version, the image base, `.text`'s RVA and size, the function count and
//! the window's low frame.
//!
//! | kind | a | b | c | d | e | f |
//! | --- | --- | --- | --- | --- | --- | --- |
//! | 0 `HIT` | function VA | thread | | | | |
//! | 1 `get()`, 3 `get(a,b)`, 4 `rand_real`, 6 `reseed` | the **caller's return address** | the `Random *` | the seed **before** the step | the caller's caller | and its caller | `arg0` |
//! | 2 `FRAME` | frame | `game_random`'s word | functions re-armed | `do_frame`'s caller | | |
//! | 5 `INFO` | code | … | | | | |
//!
//! Two things about that table decide everything below. The `Random *` in
//! `b` is what separates the **sync stream** from the renderer's four
//! other generators — only a step of `game_random` is simulation state.
//! And the seed in `c` is the word *before* the step, so a run of draws
//! reads as the LCG's own sequence and the first of them is the word to
//! seed a replay with.

use std::path::Path;

/// The image base every address here is normalised to. A run that
/// relocated reports its own base in the header and is folded back to this
/// one, so a site is comparable between runs and against the Ghidra
/// export.
pub const IMAGE_BASE: u32 = 0x0040_0000;

/// `game_random`'s RVA — the one generator whose steps are simulation
/// state. `tools/trace/report.py` carries the same constant.
pub const RVA_GAME_RANDOM: u32 = 0x00a3_7a8c;

/// The header's `kind`: `RONT`, little-endian.
const MAGIC: u32 = 0x544e_4f52;

/// The record kinds that step a generator.
const DRAW_KINDS: [u32; 4] = [1, 3, 4, 6];

/// One draw, as the trace records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    /// The **return address** into the function that drew, normalised to
    /// [`IMAGE_BASE`] — what `report.py` prints as `Class::method+0xNN`.
    pub site: u32,
    /// The `Random *` this stepped, normalised. [`Draw::sync`] is the test
    /// that matters.
    pub rng: u32,
    /// The generator's word **before** the step.
    pub seed: u32,
    /// The caller's caller, and its caller — two more frames of the `ebp`
    /// chain, normalised. Zero where the walk ran out.
    pub up: [u32; 2],
    /// The sim-frame, as `Game::do_frame` counts it; −1 for the setup path.
    pub frame: i64,
    /// The record kind — 1 `get()`, 3 `get(a, b)`, 4 `rand_real`,
    /// 6 `reseed`.
    pub kind: u32,
}

impl Draw {
    /// Whether this stepped `game_random`, which is the whole of the
    /// simulation's stream. A draw that did not is the renderer's and
    /// belongs to no frame's count.
    pub fn sync(&self) -> bool {
        self.rng == IMAGE_BASE + RVA_GAME_RANDOM
    }
}

/// A parsed `rontrace.log`.
#[derive(Clone, Debug)]
pub struct Trace {
    /// The image base the run reported.
    pub base: u32,
    /// Every draw, in file order.
    pub draws: Vec<Draw>,
    /// Each `FRAME` record: the sim-frame and `game_random`'s word at its
    /// `do_frame` entry — the same pairing `gamelog::Log::frame_seeds`
    /// gives from a `DUMP_ALL` dump, from the other side.
    pub frames: Vec<(i64, u32)>,
}

impl Trace {
    /// Parses the bytes. `None` if the header is not a trace.
    pub fn parse(bytes: &[u8]) -> Option<Trace> {
        if bytes.len() < 32 {
            return None;
        }
        let word = |off: usize| -> u32 {
            u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
        };
        if word(0) != MAGIC {
            return None;
        }
        let base = word(8);
        // A run that loaded elsewhere is folded back, so a site is the
        // same number as the Ghidra export's.
        let norm = |va: u32| -> u32 {
            if va == 0 {
                0
            } else {
                va.wrapping_sub(base).wrapping_add(IMAGE_BASE)
            }
        };
        let mut t = Trace {
            base,
            draws: Vec::new(),
            frames: Vec::new(),
        };
        let mut off = 32;
        // `emit` writes the frame counter into slot 7, and it is `-1`
        // before the first `do_frame`.
        while off + 32 <= bytes.len() {
            let r: Vec<u32> = (0..8).map(|i| word(off + i * 4)).collect();
            let frame = i64::from(r[7] as i32);
            if r[0] == 2 {
                t.frames.push((i64::from(r[1] as i32), r[2]));
            } else if DRAW_KINDS.contains(&r[0]) {
                t.draws.push(Draw {
                    site: norm(r[1]),
                    rng: norm(r[2]),
                    seed: r[3],
                    up: [norm(r[4]), norm(r[5])],
                    frame,
                    kind: r[0],
                });
            }
            off += 32;
        }
        Some(t)
    }

    /// Reads a trace off disk. `Ok(None)` when the file is not one.
    pub fn read(path: &Path) -> std::io::Result<Option<Trace>> {
        Ok(Trace::parse(&std::fs::read(path)?))
    }

    /// The sync-stream draws of one sim-frame, in the order they were
    /// taken. `-1` is the setup path.
    pub fn frame_draws(&self, frame: i64) -> Vec<Draw> {
        self.draws
            .iter()
            .filter(|d| d.frame == frame && d.sync())
            .copied()
            .collect()
    }

    /// The sync-stream draws of one sim-frame whose **site** lies inside
    /// `[lo, hi)` — one function's own draws, with everything its callees
    /// took left out. `hi` is the next function's address in the export.
    pub fn run_in(&self, frame: i64, lo: u32, hi: u32) -> Vec<Draw> {
        self.frame_draws(frame)
            .into_iter()
            .filter(|d| (lo..hi).contains(&d.site))
            .collect()
    }

    /// A frame's sync draws folded into consecutive runs of one site —
    /// `5f6446 ×3, 5f6468, 5f665c ×4, …` — which is the harness's answer
    /// to `report.py … sites` for a *sequence* rather than a total, and
    /// the shape `Built::phase_fold` prints for our own side.
    ///
    /// Sites are bare addresses: naming them needs the Ghidra export's
    /// `INDEX.tsv`, which never enters this repo. `report.py` is where a
    /// name comes from.
    pub fn site_fold(&self, frame: i64) -> String {
        let draws = self.frame_draws(frame);
        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < draws.len() {
            let mut j = i;
            while j + 1 < draws.len() && draws[j + 1].site == draws[i].site {
                j += 1;
            }
            let n = j - i + 1;
            out.push(if n > 1 {
                format!("{:x} ×{n}", draws[i].site)
            } else {
                format!("{:x}", draws[i].site)
            });
            i = j + 1;
        }
        out.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One header and three records, hand-built: a `FRAME`, a
    /// `game_random` draw and one of the renderer's.
    fn bytes(recs: &[[u32; 8]]) -> Vec<u8> {
        let mut v = Vec::new();
        for r in recs {
            for w in r {
                v.extend_from_slice(&w.to_le_bytes());
            }
        }
        v
    }

    const GAME_RANDOM: u32 = IMAGE_BASE + RVA_GAME_RANDOM;

    #[test]
    fn a_trace_parses_and_keeps_only_the_sync_stream() {
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 48_233, 0, 0xffff_ffff],
            [2, 0, 0x1111_1111, 3, 0, 0, 0, 0],
            [3, 0x005f_6446, GAME_RANDOM, 0x9c59_1b2b, 0, 0, 0, 0],
            // The renderer's — a different `Random *`, so not the stream.
            [3, 0x0022_3908, 0x0022_3908, 0xdead_beef, 0, 0, 0, 0],
            [3, 0x005f_6468, GAME_RANDOM, 0x70fc_74f0, 0, 0, 0, 0],
            [0, 0x005f_6010, 7, 0, 0, 0, 0, 0],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.base, IMAGE_BASE);
        assert_eq!(t.frames, vec![(0, 0x1111_1111)]);
        assert_eq!(t.draws.len(), 3, "the HIT and the FRAME are not draws");
        let f0 = t.frame_draws(0);
        assert_eq!(f0.len(), 2, "the renderer's is not the sync stream");
        assert_eq!(f0[0].seed, 0x9c59_1b2b);
        assert_eq!(f0[1].site, 0x005f_6468);
    }

    /// A run that loaded somewhere other than `0x400000` folds back, so a
    /// site is the same number as the export's.
    #[test]
    fn a_relocated_run_normalises_to_the_export_s_addresses() {
        let base = 0x0100_0000;
        let log = bytes(&[
            [MAGIC, 1, base, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            [3, base + 0x001f_6446, base + RVA_GAME_RANDOM, 7, 0, 0, 0, 0],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.draws[0].site, 0x005f_6446);
        assert!(t.draws[0].sync(), "and it is still the sync stream");
    }

    /// `run_in` isolates one function's own draws, and `site_fold` reads
    /// them as runs.
    #[test]
    fn run_in_isolates_a_function_and_site_fold_reads_it_as_runs() {
        let d = |site: u32| [3, site, GAME_RANDOM, 0, 0, 0, 0, 0];
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            d(0x005f_6446),
            d(0x005f_6446),
            d(0x005f_6468),
            // A callee's, outside `think_scout`'s range.
            d(0x005f_2800),
            d(0x005f_665c),
        ]);
        let t = Trace::parse(&log).expect("a trace");
        let mine = t.run_in(0, 0x005f_6010, 0x005f_6e40);
        assert_eq!(
            mine.iter().map(|d| d.site).collect::<Vec<_>>(),
            vec![0x005f_6446, 0x005f_6446, 0x005f_6468, 0x005f_665c],
            "the callee's draw is not this function's"
        );
        assert_eq!(t.site_fold(0), "5f6446 ×2, 5f6468, 5f2800, 5f665c");
    }

    #[test]
    fn something_that_is_not_a_trace_is_refused() {
        assert!(Trace::parse(b"not a trace at all, no header here").is_none());
        assert!(Trace::parse(b"short").is_none());
    }
}
