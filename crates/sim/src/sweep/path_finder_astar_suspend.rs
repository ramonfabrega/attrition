//! `PathFinder::astar_path@00683770` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::astar_path@00683770`, run by `tools/emu/sweep_path_finder_astar_suspend.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0, p1..p9 the place(s), in flag order, p10..p17 the stubs' answer window(s); the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00c0aedc points at 16 random bytes; the global cell 0x00c0af84 points at 16 random bytes; the global cell 0x00c0afa0 points at 16 random bytes; argument 0 names 16 random bytes; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x00479540 (0 args) stubbed; callee 0x004796f0 (2 args) stubbed; callee 0x00479920 (2 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006b4830 (2 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x00687970 (0 args) stubbed; callee 0x0047a370 (0 args) stubbed; callee 0x0047a2c0 (0 args) stubbed; callee 0x0047a160 (0 args) stubbed; callee 0x0047a210 (0 args) stubbed; callee 0x0047a420 (0 args) stubbed; the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `m+0x54` points at 512 random bytes less `0` (placed); the word at `p2+0x18` points at 768 random bytes less `0` (placed); the word at `p2+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0x40` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 4096 random bytes less `0` (placed); the word at `c0+0x138` points at 8192 random bytes less `0` (placed); stub 1 answered from [10, 10, 0, 3, 1] (90%); stub 2 answered from [2, 21, 0, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1] (90%); stub 4 answers the address of 32 random bytes, a `p` window after the places; stub 5 answers the address of 96 random bytes, a `p` window after the places; stub 6 answers the address of 36 random bytes, a `p` window after the places; stub 12 answers the address of 36 random bytes, a `p` window after the places; stub 0 answers a window of its own to each call, in call order (-32 bytes, ...); stub 11 answers a window of its own to each call, in call order (-16 bytes, ...); the word at `c2+0` laid as the address of `p2` every row; the word at `c3+0` laid as the address of `p2` every row; the word at `c4+0` laid as the address of `p2` every row; the word at `c5+0` laid as the address of `p2` every row; stub 9 answers 2 values in call order, printed after the answers; stub 9 answers 0..1 every row; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_astar_suspend.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**, `crate::astar_view`:
                // the entry, the fresh start and the suspend, which
                // `Sim::astar_path` runs over its `SimAstar`; here the view
                // is the emulated original's windows. Slice B: a fresh unit
                // search whose first node suspends it (`+0x64` set, `+0x80`
                // negative, `anti` 0, the node far from the goal), so every
                // row runs from the entry to the `-1` return.
                //
                // `a`: the three stack arguments (the `Stack<PathData>*`, the
                // step, `anti`), then the eighteen stubs' single answers
                // (`a[3 + k]`, stub k): 0 `get_action`, 1 its vslot `+0x10`,
                // 2 `order_type`, 3 the order's vslot `+0x14`, 4
                // `update_order`, 5 its vslot `+0x40`, 6 `Recycler<PathNode>::
                // pop`, 7/8 the two `ordered_insert`s, 9 `get_tregion` (its
                // two answers `a[21..23]`), 10 `is_ocean`, 11 `get_order`,
                // 12 `first_open_node`, 13..17 the five recyclers' pops
                // (`a[16..21]`); `a[23..27]` the null flags of the four
                // nullable answers (`get_action` twice, `get_order` twice).
                // Windows: `c0` the world (`+0` width, `+0x18` width in tiles,
                // `+0x134` the region table, `+0x138` the tile table), `c1`
                // `div_3_table`, `c2..c5` the `units` rows the unit is laid
                // in; `p0` the stack (`+0` list, `+8` length), `p1` its list,
                // `p2` the unit data (the `units` rows point back at it), `p3`
                // its type, `p4` the order list's end node, `p5` its first
                // node (`+8` the order, `+0xc` its flag), `p6` the order, `p7`
                // the open tree (`+8` length, `+0x10` current node), `p8` the
                // region table, `p9` the tile table, `p10`/`p11` the two
                // `get_action` answers, `p12` `update_order`'s, `p13` the move
                // order its slot answers (`+0x4c`/`+0x4e`), `p14` the node
                // `pop` answers, `p15`/`p16` the two `get_order` answers
                // (`+4` the flags), `p17` the node `first_open_node` answers.
                use crate::Pos;
                use crate::astar_view::{
                    AstarView, NoDangerView, Stash, SuspendView, Top, arrive, entry_steps,
                    fresh_steps, suspend_steps, top_of_loop,
                };
                use crate::orders::PathData;
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes(
                        w[o..o + 2].try_into().expect("two bytes"),
                    ))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    actions: usize,
                    tregions: usize,
                    ins_tree: Vec<String>,
                    ins_br: Vec<String>,
                }
                impl NoDangerView for V<'_> {
                    fn action_type(&mut self) -> Option<i32> {
                        let flag = self.a[23 + self.actions];
                        self.actions += 1;
                        (flag == 0).then_some(self.a[3 + 1])
                    }
                    fn owner(&mut self) -> i32 {
                        i32::from(self.p[2][9])
                    }
                    fn order_type(&mut self) -> i32 {
                        self.a[3 + 2]
                    }
                }
                impl AstarView for V<'_> {
                    fn masks2(&mut self) -> i32 {
                        rd32(&self.p[2], 0x6c)
                    }
                    fn collision(&mut self) -> i32 {
                        rd32(&self.p[3], 0x248)
                    }
                    fn transport_type(&mut self) -> bool {
                        self.p[3][0x2b4] & 0x10 != 0
                    }
                    fn ai_bit(&mut self) -> bool {
                        rd32(&self.p[2], 0x68) & 0x40000 != 0
                    }
                    fn list_present(&mut self) -> bool {
                        rd32(&self.p[2], 0xdc) != 0
                    }
                    fn cursor_to_front(&mut self) {
                        // `+0xd4 = end->next`, `+0xcc = its order`, `+0xd0 = its flag`.
                        let n = rd32(&self.p[4], 4);
                        let o = rd32(&self.p[5], 8);
                        let f = self.p[5][0xc];
                        wr32(&mut self.p[2], 0xd4, n);
                        wr32(&mut self.p[2], 0xcc, o);
                        self.p[2][0xd0] = f;
                    }
                    fn has_order(&mut self) -> bool {
                        rd32(&self.p[2], 0xcc) != 0
                    }
                    fn order_is_move(&mut self) -> bool {
                        self.a[3 + 3] != 0
                    }
                    fn move_offset(&mut self) -> (i32, i32) {
                        (rd16(&self.p[13], 0x4c), rd16(&self.p[13], 0x4e))
                    }
                    fn order_present(&mut self) -> bool {
                        self.a[25] == 0
                    }
                    fn order_flags(&mut self) -> i32 {
                        // `get_order` asked again: a null answer reads the
                        // zero page.
                        if self.a[26] == 0 {
                            i32::from(self.p[16][4])
                        } else {
                            0
                        }
                    }
                    fn world_width(&mut self) -> i32 {
                        rd32(&self.c[0], 0)
                    }
                    fn world_width_tiles(&mut self) -> i32 {
                        rd32(&self.c[0], 0x18)
                    }
                    fn div3(&mut self, i: i32) -> i32 {
                        rd32(&self.c[1], 4 * i as usize)
                    }
                    fn region_of_cell(&mut self, cx: i32, cy: i32) -> i32 {
                        let w = rd32(&self.c[0], 0);
                        rd16(&self.p[8], 4 + (cy * w + cx) as usize * 0x1c) & 0xffff
                    }
                    fn tregion(&mut self, _tx: i32, _ty: i32) -> i32 {
                        let v = self.a[21 + self.tregions.min(1)];
                        self.tregions += 1;
                        v
                    }
                    fn is_ocean_cell(&mut self, _cx: i32, _cy: i32) -> bool {
                        self.a[3 + 10] != 0
                    }
                    fn tile_mask(&mut self, tx: i32, ty: i32) -> i32 {
                        let w = rd32(&self.c[0], 0x18);
                        i32::from(u16::from_le_bytes(
                            self.p[9][(ty * w + tx) as usize * 2..][..2]
                                .try_into()
                                .expect("two bytes"),
                        ))
                    }
                    fn pop_path(&mut self) -> PathData {
                        let mut len = rd32(&self.p[0], 8);
                        if len < 1 {
                            len = 1;
                        }
                        len -= 1;
                        wr32(&mut self.p[0], 8, len);
                        let k = len as usize * 16;
                        PathData {
                            to: Pos::new(rd32(&self.p[1], k), rd32(&self.p[1], k + 4)),
                            tolerance: rd32(&self.p[1], k + 8),
                            flags: rd32(&self.p[1], k + 12) as u8,
                        }
                    }
                    fn top_tolerance(&mut self) -> i32 {
                        let len = rd32(&self.p[0], 8);
                        if len == 0 {
                            0
                        } else {
                            rd32(&self.p[1], (len as usize - 1) * 16 + 8)
                        }
                    }
                    fn set_iroquois(&mut self, word: i32) {
                        wr32(self.m, 0x74, word);
                    }
                    fn set_no_danger(&mut self, on: bool) {
                        wr32(self.m, 0x7c, i32::from(on));
                    }
                    fn reset_valid_hit(&mut self) {
                        wr32(self.m, 0x90, 0);
                    }
                    fn seed_open(&mut self, start: Pos, metric: i32, estimate: i32) {
                        // The node `pop` answered (`p14`), field by field.
                        let n = &mut self.p[14];
                        wr32(n, 0, start.x);
                        wr32(n, 4, start.y);
                        wr32(n, 8, 0);
                        wr32(n, 0x14, 0);
                        wr32(n, 0x20, 0);
                        n[0x1e] = 0;
                        n[0x1f] = 0;
                        wr32(n, 0x18, metric);
                        wr32(n, 0xc, estimate);
                        wr32(n, 0x10, estimate);
                        self.ins_tree.push(format!("p7+0x0,p14+0x0,{estimate}"));
                        let cur = rd32(&self.p[7], 0x10);
                        let ecx = rd32(self.m, 0x44);
                        self.ins_br.push(format!("{ecx},{cur},{metric}"));
                    }
                    fn set_avoid(&mut self, land: i32, sea: i32) {
                        wr32(self.m, 0x88, land);
                        wr32(self.m, 0x8c, sea);
                    }
                }
                impl SuspendView for V<'_> {
                    fn reinsert_current(&mut self) {
                        let (value, metric) = (rd32(&self.p[17], 0x10), rd32(&self.p[17], 0x18));
                        self.ins_tree.push(format!("p7+0x0,p17+0x0,{value}"));
                        let cur = rd32(&self.p[7], 0x10);
                        let ecx = rd32(self.m, 0x44);
                        self.ins_br.push(format!("{ecx},{cur},{metric}"));
                    }
                    fn hand_over(&mut self, st: &Stash) {
                        wr32(self.m, 0x84, 1);
                        // Each container goes to the unit and a fresh one
                        // comes off its recycler (stubs 13..17).
                        let m = &mut *self.m;
                        let u = &mut self.p[2];
                        for (k, (mine, theirs)) in
                            [(0x40, 0x104), (0x44, 0x108), (0x48, 0x10c), (0x50, 0x110)]
                                .into_iter()
                                .enumerate()
                        {
                            wr32(u, theirs, rd32(m, mine));
                            wr32(m, mine, self.a[3 + 13 + k]);
                        }
                        wr32(u, 0x134, rd32(m, 0x90));
                        wr32(u, 0x114, rd32(m, 0x4c));
                        wr32(m, 0x4c, self.a[3 + 17]);
                        wr32(u, 0x128, st.tol);
                        wr32(u, 0x12c, st.pref);
                        wr32(u, 0x140, st.goal.x);
                        wr32(u, 0x144, st.goal.y);
                        wr32(u, 0x130, st.start_dist);
                        wr32(u, 0x148, st.traversed);
                        wr32(u, 0x138, rd32(m, 0x88));
                        wr32(u, 0x13c, rd32(m, 0x8c));
                    }
                }
                let (step, anti) = (a[1], a[2]);
                assert_eq!(rd32(m, 0x84), 0, "slice B is a fresh search");
                let anti_unit = rd32(m, 0x64) != 0;
                let limit = rd32(m, 0x80);
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    actions: 0,
                    tregions: 0,
                    ins_tree: Vec::new(),
                    ins_br: Vec::new(),
                };
                let e = entry_steps(&mut v, step, anti);
                let f = fresh_steps(&mut v, &e, step);
                // The loop's head: `first_open_node` answers `p17`.
                let (cx, cy) = (rd32(&v.p[17], 0), rd32(&v.p[17], 4));
                let manh = cx
                    .wrapping_sub(f.goal.x)
                    .wrapping_abs()
                    .wrapping_add(cy.wrapping_sub(f.goal.y).wrapping_abs());
                let top = top_of_loop(
                    manh,
                    arrive(f.tol, e.stride),
                    0,
                    0,
                    e.work_cap,
                    anti_unit,
                    limit,
                    anti,
                );
                assert_eq!(top, Top::Suspend, "the draw reaches only the suspend");
                suspend_steps(
                    &mut v,
                    &Stash {
                        tol: f.tol,
                        pref: f.pref,
                        goal: f.goal,
                        start_dist: f.start_dist,
                        traversed: 0,
                        avoid: f.avoid,
                    },
                );
                log.push(v.ins_tree.join("/"));
                log.push(v.ins_br.join("/"));
                -1
            }
        };
        assert_eq!(got, want_eax, "astar_path — eax, row `{line}`");
        assert_eq!(log, want_log, "astar_path — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "astar_path — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "astar_path — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "astar_path — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("PathFinder::astar_path@00683770: {n} rows agree");
}
