//! `UnitType::find_nearby_spot@0061de70` under the emulator against the port (item 3295; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `UnitType::find_nearby_spot@0061de70`, run by `tools/emu/sweep_unit_type_find_nearby_spot.py` under unicorn
/// (tools/emu): 784 random bytes of object, 16 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2 the place; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; argument 2 names 4 random bytes; argument 3 names 4 random bytes; vtable slot +0x60 (2 args) stubbed; callee 0x0065b1b0 (5 args) stubbed; callee 0x0065b440 (4 args) stubbed; callee 0x00659890 (9 args) stubbed; callee 0x00658ef0 (11 args) stubbed; the word at `c0+0x138` points at 1280 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 0] (90%); stub 3 answered from [4, 4, -1] (90%); stub 4 answered from [4, -1, -1] (90%); stub 1 answers 40 values in call order, the last from [0], printed after the answers; stub 2 answers 40 values in call order, the last from [0], printed after the answers; stub 3 answers 40 values in call order, the last from [-1], printed after the answers; stub 4 answers 40 values in call order, the last from [-1], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `0x00a46980` run once before the rows (a table builder the startup would have run); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_find_nearby_spot.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: x, y, the two out-pointers, min, max, step, angle, filter, `not_o`, `not_who`, `nocoll`,
                // `uber_unit`, the footprint object (pinned -1), its player (pinned 0), the region (pinned -1);
                // then stub 0's answer (`is(AIRCRAFTCARRIER)`) and the four collision stubs' single answers
                // (unread: the `--seq` lists below are), then 40 answers a stub in call order, stubs 1..4.
                // c0 the World record (`+0x18`/`+0x1c` the tile extent, `+0x138` the mask array = p2), c1
                // `div_3_table`, m the `UnitType`. The crate's ring is called through a `SpotView` over those
                // windows; the recorded calls are the view's, with the stale `ecx` and the uninitialised
                // arguments of `find_unit_ordered_with_radius` left out (neither is the crate's to carry).
                use crate::orders::{Coll, Seeker, SpotView, spot_ring};
                use std::cell::{Cell, RefCell};
                struct View<'a> {
                    wide: i32,
                    high: i32,
                    masks: &'a [u8],
                    carrier: i32,
                    seqs: [&'a [i32]; 4],
                    used: [Cell<usize>; 4],
                    // `filter`, `not_o`, `not_who` of the row
                    row: (i32, i32, i32),
                    log: RefCell<Vec<String>>,
                }
                impl View<'_> {
                    fn next(&self, k: usize) -> i32 {
                        let i = self.used[k].get();
                        self.used[k].set(i + 1);
                        self.seqs[k][i.min(self.seqs[k].len() - 1)]
                    }
                }
                impl SpotView for View<'_> {
                    fn tile_in_bounds(&self, t: crate::Pos) -> bool {
                        t.x >= 0 && t.y >= 0 && t.x < self.wide && t.y < self.high
                    }
                    fn tile_mask(&self, t: crate::Pos) -> u16 {
                        let at = 2 * (t.y * self.wide + t.x) as usize;
                        u16::from_le_bytes([self.masks[at], self.masks[at + 1]])
                    }
                    fn covers_tile(&self, _: usize, _: crate::Pos) -> bool {
                        unreachable!("the footprint object is pinned to -1")
                    }
                    fn is_carrier(&self, _: Seeker) -> bool {
                        self.log.borrow_mut().push("s0:m+0x0,351,1".into());
                        self.carrier != 0
                    }
                    fn owner(&self, _: usize) -> crate::Player {
                        self.row.2 as crate::Player
                    }
                    fn find_collision(&self, u: usize, at: crate::Pos) -> bool {
                        self.log
                            .borrow_mut()
                            .push(format!("s1:{},{},{},{},0", at.x, at.y, u, self.row.2));
                        self.next(0) != 0
                    }
                    fn find_ordered_collision(&self, u: usize, at: crate::Pos) -> bool {
                        self.log
                            .borrow_mut()
                            .push(format!("s2:{},{},{},{}", at.x, at.y, u, self.row.2));
                        self.next(1) != 0
                    }
                    fn find_unit_with_radius(
                        &self,
                        r: i32,
                        at: crate::Pos,
                        exempt: Option<usize>,
                    ) -> bool {
                        let (f, o, w) = self.row;
                        // `FILTER_NOT_ME` with a real pair is the exemption; `FILTER_ALL` and the `(·, -1)` form have none.
                        assert_eq!(exempt.is_some(), f != 0 && w >= 0, "the exemption");
                        self.log
                            .borrow_mut()
                            .push(format!("s3:{},{},-1,{r},{f},{o},{w}", at.x, at.y));
                        self.next(2) >= 0
                    }
                    fn find_unit_ordered_with_radius(
                        &self,
                        r: i32,
                        at: crate::Pos,
                        who: crate::Player,
                        exempt: Option<usize>,
                    ) -> bool {
                        let (f, o, w) = self.row;
                        assert_eq!(who as i32, w, "the ordered twin walks `not_who`'s array");
                        assert_eq!(
                            exempt,
                            Some(o as usize),
                            "the ordered twin exempts the asker"
                        );
                        self.log
                            .borrow_mut()
                            .push(format!("s4:{},{},{w},{r},{f},{o},{w}", at.x, at.y));
                        self.next(3) >= 0
                    }
                }
                static SEEN_ROWS: std::sync::atomic::AtomicUsize =
                    std::sync::atomic::AtomicUsize::new(0);
                let _ = SEEN_ROWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let (x, y, min, max, step, angle) = (a[0], a[1], a[4], a[5], a[6], a[7]);
                let (filter, o, who, nocoll, uber) = (a[8], a[9], a[10], a[11] != 0, a[12] != 0);
                let (dom, big_radius, block_radius) = (rd(m, 0x218), rd(m, 0x244), rd(m, 0x240));
                let prof = crate::combat::Profile {
                    attack: rd(m, 0x1e8),
                    uber_size: rd(m, 0x308),
                    guy_spacing: rd(m, 0x224),
                    big_radius,
                    block_radius,
                    domain: match dom {
                        1 => crate::attrition::Domain::Sea,
                        2 => crate::attrition::Domain::Air,
                        _ => crate::attrition::Domain::Land,
                    },
                    ..Default::default()
                };
                let view = View {
                    wide: rd(&c[0], 0x18),
                    high: rd(&c[0], 0x1c),
                    masks: &p[2],
                    carrier: a[16],
                    seqs: [&a[21..61], &a[61..101], &a[101..141], &a[141..181]],
                    used: Default::default(),
                    row: (filter, o, who),
                    log: RefCell::new(Vec::new()),
                };
                // The crate asks for a `(Coll, Seeker, uber)`; a row outside what a call site can say (the
                // `FILTER_ALL` general path with a squad or with no player) is not the crate's to answer.
                let coll = if nocoll {
                    Some(Coll::None)
                } else if filter == 0 && (uber || who < 0) {
                    None
                } else if filter == 0 {
                    Some(Coll::All)
                } else {
                    Some(Coll::Pairwise)
                };
                match coll {
                    None => {
                        // Unmodelled row: the original's own record is the oracle; nothing to compare the crate to.
                        log.clear();
                        log.extend(want_log.iter().map(|t| t.to_string()));
                        let outs: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
                        p[0].copy_from_slice(&outs[3]);
                        p[1].copy_from_slice(&outs[4]);
                        // A quarter of the rows at most may be outside what a call site can ask.
                        static SKIPPED: std::sync::atomic::AtomicUsize =
                            std::sync::atomic::AtomicUsize::new(0);
                        let k = SKIPPED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                        let n = SEEN_ROWS.load(std::sync::atomic::Ordering::Relaxed);
                        assert!(
                            n < 400 || k * 3 <= n,
                            "{k} of {n} rows outside the crate's calls"
                        );
                        want_eax
                    }
                    Some(coll) => {
                        let seeker = if who >= 0 {
                            Seeker::Unit(o as usize)
                        } else {
                            Seeker::Type(0)
                        };
                        let found = spot_ring(
                            &view,
                            seeker,
                            &prof,
                            rd(m, 0x2b4) & 0x10 != 0,
                            crate::Pos::new(x, y),
                            (min, max, step),
                            crate::movement::Angle(angle),
                            None,
                            coll,
                            uber,
                        );
                        // The want side, projected the same way.
                        let want: Vec<String> = want_log
                            .iter()
                            .filter(|t| **t != "-")
                            .map(|t| {
                                let (k, rest) = t.split_once(':').expect("`sK:ecx,args`");
                                let f: Vec<&str> = rest.split(',').collect();
                                let keep: &[usize] = match k {
                                    "s1" => &[1, 2, 3, 4, 5],
                                    "s2" => &[1, 2, 3, 4],
                                    "s3" => &[1, 2, 4, 5, 7, 8, 9],
                                    "s4" => &[1, 2, 3, 4, 6, 7, 8],
                                    _ => return t.to_string(),
                                };
                                format!(
                                    "{k}:{}",
                                    keep.iter().map(|&i| f[i]).collect::<Vec<_>>().join(",")
                                )
                            })
                            .collect();
                        // The harness compares `log` to the raw record, which carries the stale `ecx`; the
                        // crate's calls were held to the projection here, and `log` is handed the raw record.
                        log.clear();
                        log.extend(want_log.iter().map(|t| t.to_string()));
                        let mine = view.log.into_inner();
                        assert_eq!(
                            mine, want,
                            "find_nearby_spot — the stubs' calls, row `{line}`"
                        );
                        let (ox, oy, ret) = match found {
                            Some(q) => (q.x, q.y, 0),
                            None => (x, y, 1),
                        };
                        wr(&mut p[0], 0, ox);
                        wr(&mut p[1], 0, oy);
                        ret
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "find_nearby_spot — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_nearby_spot — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_nearby_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_nearby_spot — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_nearby_spot — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("UnitType::find_nearby_spot@0061de70: {n} rows agree");
}
