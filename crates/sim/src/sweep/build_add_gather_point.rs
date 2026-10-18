//! `Build::add_gather_point@00622e70` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

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

/// `Build::add_gather_point@00622e70`, run by `tools/emu/sweep_build_add_gather_point.py` under unicorn
/// (tools/emu): 256 random bytes of object, 4 argument(s), `p` windows in order: p0..p25 the place(s), in flag order; the global cell 0x00c0618c points at 384 random bytes; the global cell 0x00c8d900 points at 32 random bytes; 4 random bytes at 0x00c8d908; 16 random bytes at 0x00e90000; 16 random bytes at 0x00e90010; 16 random bytes at 0x00e90020; 16 random bytes at 0x00e90030; 16 random bytes at 0x00e90040; 16 random bytes at 0x00e90050; 16 random bytes at 0x00e90100; 16 random bytes at 0x00e90110; 16 random bytes at 0x00e91000; 16 random bytes at 0x00e91010; 16 random bytes at 0x00e91020; 16 random bytes at 0x00e91030; 16 random bytes at 0x00e91040; 16 random bytes at 0x00e91050; 16 random bytes at 0x00e91100; callee 0x00623180 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x00470c60 (1 args) run, not hooked; callee 0x00609dc0 (1 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x005e48c0 (7 args) stubbed; callee 0x005e4350 (6 args) stubbed; callee 0x0046edd0 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x30 (0 args) stubbed; vtable slot +0x84 (0 args) stubbed; vtable slot +0x8c (0 args) stubbed; the word at `0xc0aec0+u8(m,9)*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p1+0x18` points at 560 random bytes less `0` (placed); the word at `p2+0x18` points at 560 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); the word at `p1+0x24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p2+0x24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p3+0x24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p1+0x28` points at 64 random bytes less `0` (placed); the word at `p2+0x28` points at 64 random bytes less `0` (placed); the word at `p3+0x28` points at 64 random bytes less `0` (placed); the word at `p10+0x20` points at 32 random bytes less `0` (placed); the word at `p11+0x20` points at 32 random bytes less `0` (placed); the word at `p12+0x20` points at 32 random bytes less `0` (placed); the word at `p10+0x24` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p11+0x24` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p12+0x24` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p16+0x14` points at 64 random bytes less `0` (placed); the word at `p17+0x14` points at 64 random bytes less `0` (placed); the word at `p18+0x14` points at 64 random bytes less `0` (placed); the word at `p16+0x30` points at 64 random bytes less `0` (placed); the word at `p17+0x30` points at 64 random bytes less `0` (placed); the word at `p18+0x30` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 32 random bytes less `0` (placed); stub 3 answered from [0, 0, 0, 0, 0, 1, -1] (90%); stub 9 answered from [1, 1, 1, 0] (90%); stub 10 answered from [1, 1, 1, 0] (90%); the stubbed vtable laid at `p25+0`; the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 1 answers 15274240..15274240 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 3 writes a value in [0, 0, 0, 1] through its argument 0, printed after the answers (`o` in a place); stub 1 is `__cdecl`: the caller pops its arguments; stub 4 answers from its own `ecx`: the word at `+0x24`; stub 5 answers from its own `ecx`: the word at `+0x28`; stub 11 answers from its own `ecx`: the word at `+0x20`; stub 12 answers from its own `ecx`: the word at `+0x20`; stub 13 answers from its own `ecx`: the word at `+0x24`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_add_gather_point.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(18);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0..4]` the point's `x`, `y`, `action` (the low byte is stored) and the queue position (2 `QUEUE_NEW`,
            // 0 `QUEUE_FIRST`, 1 `QUEUE_LAST`); `a[4 + i]` stub `i`'s answer, `a[18]` the player stub 3 writes through its
            // argument. Stubs: 0 `clear_gather`, 1 `malloc` (always the point at `0xe91100`, `c[17]`), 2 `LinkListBase::add`
            // (run, not hooked: with `Recycler::pop` over `c[1]`, `c[2]`), 3 `UnitData::home_base`, 4 `get_order`, 5
            // `update_order`, 6 `add_strafe_order`, 7 `add_air_patrol_order`, 8 `SimpleArray::make_valid`, 9 and 10 the
            // `is(0x1bf, 1)` asks (the type record's slot `+0x60` on the `Build::vftable` arm, else `this`'s `+0xb8`), 11 the
            // order's `is_air`, 12 and 13 `update_order()`'s slots `+0x84` (the heights) and `+0x8c` (the patrol).
            // Windows: `m` the building; `c[0]` the objects singleton (`+0x15c` the unit counts), `c[1]` the pool's array
            // (`c[2]` its count), `c[3..9]` list nodes at `0xe90000 + 0x10 k`, `c[9..11]` the pool's nodes, `c[11..17]` the
            // nodes' points at `0xe91000 + 0x10 k`, `c[17]` the new point; `p[0]` the owner's units array, `p[1..4]` the
            // units, `p[4..7]` their types, `p[7..10]` their orders, `p[10..13]` their `update_order()`s, `p[13..16]` the
            // heights, `p[16..19]` the patrols (`p[19..22]` the x arrays, `p[22..25]` the y arrays), `p[25]` this's type.
            //
            // **The port is the crate's own steps**, `crate::rally::add_gather_point_steps`, which `Sim::add_gather_point`
            // calls over its `GatherPointView`; here the view is the emulated original's windows. The list is the
            // original's circular `LinkListBase` laid in the windows (its cursor and length with it: this crate keeps a
            // `Vec`), and the sixth argument of `add_air_patrol_order` is a stale `ecx` the view tracks (`stale`).
            use crate::orders::QueuePos;
            use crate::rally::{GatherPoint, GatherPointView, add_gather_point_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                stale: String,
            }
            /// The window of the table at an image address, and its name.
            fn cw(va: i32) -> usize {
                let va = va as u32;
                match va {
                    0xe90000..=0xe9005f => 3 + ((va - 0xe90000) / 16) as usize,
                    0xe90100 => 9,
                    0xe90110 => 10,
                    0xe91000..=0xe9105f => 11 + ((va - 0xe91000) / 16) as usize,
                    0xe91100 => 17,
                    _ => panic!("address {va:#x} is no table"),
                }
            }
            /// The table `k`'s image address.
            fn tab(k: usize) -> u32 {
                match k {
                    3..=8 => 0xe90000 + 0x10 * (k as u32 - 3),
                    9 | 10 => 0xe90100 + 0x10 * (k as u32 - 9),
                    11..=16 => 0xe91000 + 0x10 * (k as u32 - 11),
                    _ => 0xe91100,
                }
            }
            /// The recorder's name for an image address: the first table with `at <= va <= at + 16`, so a node that
            /// starts where another ends is named by that one (`c3+0x10` for `c4`).
            fn tok(va: i32) -> String {
                let k = (3..18)
                    .find(|&k| (tab(k)..=tab(k) + 16).contains(&(va as u32)))
                    .expect("a table");
                format!("c{k}+{:#x}", va as u32 - tab(k))
            }
            impl V<'_> {
                fn player(&self) -> usize {
                    usize::from(self.m[9])
                }
                fn id(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                }
            }
            impl GatherPointView for V<'_> {
                fn clear_gather(&mut self) {
                    self.log.push("s0:m+0x0".into());
                }
                fn add(&mut self, q: GatherPoint, pos: QueuePos) {
                    // `malloc(0x10)` answers `c[17]`: the `GatherPoint`.
                    let pt = &mut self.c[17];
                    wr(pt, 0, 0xb42168u32 as i32);
                    wr(pt, 4, q.pos.x);
                    wr(pt, 8, q.pos.y);
                    pt[0xc] = q.action;
                    // `Recycler::pop`, then `LinkListBase::add`.
                    let n = rd(&self.c[2], 0);
                    let n = n.max(1) - 1;
                    wr(&mut self.c[2], 0, n);
                    let node = rd(&self.c[1], 4 * n as usize);
                    let nw = cw(node);
                    self.c[nw][0xc] = 0;
                    wr(&mut self.c[nw], 8, 0xe91100);
                    wr(&mut self.c[nw], 4, 0);
                    wr(&mut self.c[nw], 0, 0);
                    let head = rd(self.m, 0xcc);
                    if head != 0 {
                        let hw = cw(head);
                        let tail = rd(&self.c[hw], 4);
                        wr(&mut self.c[nw], 0, head);
                        wr(&mut self.c[nw], 4, tail);
                        let tw = cw(tail);
                        wr(&mut self.c[tw], 0, node);
                        wr(&mut self.c[hw], 4, node);
                    } else {
                        wr(&mut self.c[nw], 0, node);
                        wr(&mut self.c[nw], 4, node);
                    }
                    wr(self.m, 0xc8, rd(self.m, 0xc8).wrapping_add(1));
                    wr(self.m, 0xcc, node);
                    wr(self.m, 0xc4, node);
                    wr(self.m, 0xbc, 0xe91100);
                    self.m[0xc0] = self.c[nw][0xc];
                    if pos == QueuePos::Last {
                        wr(self.m, 0xcc, rd(&self.c[nw], 0));
                    }
                }
                fn hangar(&mut self) -> bool {
                    if rd(self.m, 0x60) & 8 == 0 {
                        return false;
                    }
                    if rd(self.m, 0) as u32 == 0xb42174 {
                        self.log.push("s9:p25+0x0,447,1".into());
                        self.a[4 + 9] != 0
                    } else {
                        self.log.push("s10:m+0x0,447,1".into());
                        self.a[4 + 10] != 0
                    }
                }
                fn planes(&mut self) -> usize {
                    rd(&self.c[0], 0x15c + 4 * self.player()).max(0) as usize
                }
                fn homed_plane(&mut self, i: usize) -> bool {
                    if rd(&self.p[1 + i], 8) & 1 == 0 || rd(&self.p[4 + i], 0x218) != 2 {
                        return false;
                    }
                    self.log.push(format!("s3:p{}+0x0,sp-0x34", 1 + i));
                    // The answer is the row's `a[7]`; the player goes out through the argument.
                    self.id() == self.a[4 + 3] && self.player() as i32 == self.a[18]
                }
                fn heights(&mut self, i: usize) -> Option<(i32, i32)> {
                    if rd(&self.p[1 + i], 0x24) == 0 || rd(&self.p[7 + i], 0x20) == 0 {
                        return None;
                    }
                    let h = &self.p[13 + i];
                    Some((rd(h, 0xc), rd(h, 0x10)))
                }
                fn points(&mut self) -> Vec<GatherPoint> {
                    let head = rd(self.m, 0xcc);
                    self.stale = tok(head);
                    let mut out = Vec::new();
                    let mut node = head;
                    loop {
                        let nw = &self.c[cw(node)];
                        let d = &self.c[cw(rd(nw, 8))];
                        out.push(GatherPoint {
                            pos: crate::Pos {
                                x: rd(d, 4),
                                y: rd(d, 8),
                            },
                            action: d[0xc],
                        });
                        node = rd(nw, 0);
                        if node == head {
                            break;
                        }
                    }
                    // The cursor ends on the head after the lap.
                    let hw = &self.c[cw(head)];
                    let (data, metric) = (rd(hw, 8), hw[0xc]);
                    wr(self.m, 0xc4, head);
                    wr(self.m, 0xbc, data);
                    self.m[0xc0] = metric;
                    out
                }
                fn strafe(&mut self, i: usize, q: GatherPoint) {
                    self.log.push(format!(
                        "s6:p{}+0x0,{},{},{},{},1,2,0",
                        1 + i,
                        q.pos.x,
                        q.pos.y,
                        self.id(),
                        self.player()
                    ));
                    self.stale = format!("p{}+0x0", 1 + i);
                }
                fn air_patrol(&mut self, i: usize, q: GatherPoint) -> bool {
                    self.log.push(format!(
                        "s7:p{}+0x0,{},{},{},{},1,{}",
                        1 + i,
                        q.pos.x,
                        q.pos.y,
                        self.id(),
                        self.player(),
                        self.stale
                    ));
                    self.stale = format!("p{}+0x0", 10 + i);
                    rd(&self.p[10 + i], 0x24) != 0
                }
                fn patrol_push(&mut self, i: usize, q: GatherPoint) {
                    let idx = rd(&self.p[16 + i], 8);
                    self.log.push(format!("s8:p{}+0x4,{idx}", 16 + i));
                    self.log.push(format!("s8:p{}+0x20,{idx}", 16 + i));
                    wr(&mut self.p[19 + i], 4 * idx as usize, q.pos.x);
                    wr(&mut self.p[22 + i], 4 * idx as usize, q.pos.y);
                    self.stale = format!("p{}+0x0", 22 + i);
                }
                fn patrol_heights(&mut self, i: usize, alt: i32, sharp: i32) {
                    wr(&mut self.p[16 + i], 0x4c, alt);
                    wr(&mut self.p[16 + i], 0x50, sharp);
                }
            }
            let pos = match a[3] {
                2 => QueuePos::New,
                0 => QueuePos::First,
                _ => QueuePos::Last,
            };
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                stale: String::new(),
            };
            let q = GatherPoint {
                pos: crate::Pos { x: a[0], y: a[1] },
                action: a[2] as u8,
            };
            add_gather_point_steps(&mut v, q, pos);
            // A row that called nothing records `-`.
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_gather_point — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_gather_point — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_gather_point — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[19 + i],
                "add_gather_point — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Build::add_gather_point@00622e70: {n} rows agree");
}
