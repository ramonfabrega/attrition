//! `PathFinder::find_upath@00682f30` under the emulator against the port (item 3339; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::find_upath@00682f30`, run by `tools/emu/sweep_path_finder_find_upath_00682f30.py` under unicorn
/// (tools/emu): 16 random bytes of object, 6 argument(s), `p` windows in order: p0 argument 0, p1..p7 the place(s), in flag order, p8..p9 the stubs' answer window(s); the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c0618c points at 112 random bytes; the global cell 0x7c01002c points at 16 random bytes; 152 random bytes at 0x00e85e40; 112 random bytes at 0x00c0aec0; 20 random bytes at 0x00ee14fc; argument 0 names 16 random bytes; callee 0x0046f960 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x00687c80 (3 args) stubbed; callee 0x00683770 (3 args) stubbed; callee 0x00687ae0 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x0046e8c0 (1 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `p0+0` points at 512 random bytes less `0` (placed); the word at `c2+0x14+0x1c*a[3]` points at 12 random bytes less `0` (placed); the word at `p2+4*a[4]` points at 32 random bytes less `0` (placed); the word at `c5+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p4+4*a[4]` points at 16 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); the word at `c6+4` points at 512 random bytes less `0` (placed); stub 0 answered from [0, 0, 1] (90%); stub 2 answered from [0, 1, 1, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 2, 3] (90%); stub 5 answers the address of 32 random bytes, a `p` window after the places; stub 9 answers the address of 88 random bytes, a `p` window after the places; the word at `c3+0` laid as the address of `c3` every row; stub 1 answers 24 values in call order, printed after the answers; stub 2 answers 12 values in call order, the last from [1], printed after the answers; stub 3 answers -1..4 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_find_upath_00682f30.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**,
                // `crate::upath::find_upath_steps`, which `Sim::upath` runs over
                // its `SimUpath`; here the view is the emulated original's
                // windows.
                //
                // `a`: the six stack arguments (the `Stack<PathData>*`, the
                // from-position `x`, `y`, the unit class and slot, `anti`), the
                // ten stubs' single answers (`a[6..16]`: 0 `can_transport`, 3
                // `astar_path`, 8 the slot `+0x14`; 5 and 9 the order windows'
                // addresses; the others unused), stub 1's twenty-four
                // `get_tregion` answers (`a[16..40]`) and stub 2's twelve
                // `valid_ucoord` answers (`a[40..52]`). Stubs: 0
                // `UnitData::can_transport`, 1 `get_tregion` (its two arguments
                // are locals; the cell pair is printed after the record), 2
                // `valid_ucoord`, 3 `astar_path`, 4 `kill_lists`, 5
                // `Unit::update_order`, 6 `kill_current_order`, 7
                // `Stack<PathData>::increase_size` (the native push and pop run
                // in the emulator; this view writes the same bytes), 8/9 the
                // slots `+0x14`, `+0x40`. Windows: `c0` `div_3_table`, `c1` the
                // world (`+0/+4` its extent in 16-tile blocks), `c2` the
                // objects singleton, `c3` the thread-local block (`+8` the init
                // epoch), `c4` the pathfinder (`+0x54` the unit, `+0x58/+0x5c`
                // the start tile, `+0x64`, `+0x68/+0x6c`, `+0x74`, `+0x84` the
                // `saving` flag, `+0x94`), `c5` the units singleton, `c6` the
                // function-static `keep` (`+0` its init guard held at or below
                // the epoch, so the init arm is not reached; `+4` list, `+8`
                // size, `+0xc` length, `+0x10` increment); `p0` the stack
                // (`+0` list, `+4` size, `+8` length, `+0xc` increment), `p1`
                // its list, `p2` the objects row, `p3` the object (`+0x10/+0x14`
                // the position xor `0x63637`, `+0x18` the type), `p4` the units
                // row, `p5` the unit, `p6` the type (`+0x218` domain), `p7`
                // `keep`'s list, `p8` the order `update_order` answers, `p9` the
                // `MoveOrder` its slot `+0x40` answers (`+0x1c` retry).
                use crate::Pos;
                use crate::orders::PathData;
                use crate::upath::{UpathView, find_upath_steps};
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn hex(v: i32) -> String {
                    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                }
                fn ent(l: &[u8], k: i32) -> PathData {
                    let o = 16 * k as usize;
                    PathData {
                        to: Pos::new(rd32(l, o), rd32(l, o + 4)),
                        tolerance: rd32(l, o + 8),
                        flags: rd32(l, o + 12) as u8,
                    }
                }
                fn s_pop(h: &mut [u8], b: usize, l: &[u8]) -> PathData {
                    let len = rd32(h, b + 8).max(1) - 1;
                    wr32(h, b + 8, len);
                    ent(l, len)
                }
                fn s_peek(h: &[u8], b: usize, l: &[u8]) -> PathData {
                    let len = rd32(h, b + 8);
                    ent(l, if len < 1 { 0 } else { len - 1 })
                }
                fn s_push(
                    h: &mut [u8],
                    b: usize,
                    l: &mut [u8],
                    e: PathData,
                    log: &mut Vec<String>,
                    name: &str,
                ) {
                    let len = rd32(h, b + 8);
                    if len >= rd32(h, b + 4) {
                        log.push(format!("s7:{name},{}", h[b + 12] as i8));
                    }
                    let o = 16 * len as usize;
                    wr32(l, o, e.to.x);
                    wr32(l, o + 4, e.to.y);
                    wr32(l, o + 8, e.tolerance);
                    wr32(l, o + 12, i32::from(e.flags));
                    wr32(h, b + 8, len + 1);
                }
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    tiles_x: Vec<String>,
                    tiles_y: Vec<String>,
                    tregions: usize,
                    valids: usize,
                }
                impl UpathView for V<'_> {
                    fn begin(&mut self, here: Pos) {
                        let unit = self.p[4][4 * self.a[4] as usize..][..4].to_vec();
                        self.c[4][0x54..0x58].copy_from_slice(&unit);
                        let sx = rd32(&self.c[0], 4 * (here.x >> 6) as usize);
                        let sy = rd32(&self.c[0], 4 * (here.y >> 6) as usize);
                        wr32(&mut self.c[4], 0x58, sx);
                        wr32(&mut self.c[4], 0x5c, sy);
                        wr32(&mut self.c[4], 0x68, 0);
                        wr32(&mut self.c[4], 0x6c, 0);
                        wr32(&mut self.c[4], 0x94, 0);
                    }
                    fn saving(&self) -> bool {
                        rd32(&self.c[4], 0x84) != 0
                    }
                    fn cell48(&self, c: i32) -> i32 {
                        rd32(&self.c[0], 4 * (c >> 4) as usize)
                    }
                    fn world_dims(&self) -> (i32, i32) {
                        (rd32(&self.c[1], 0), rd32(&self.c[1], 4))
                    }
                    fn pop(&mut self) -> Option<PathData> {
                        let (h, l) = self.p.split_at_mut(1);
                        Some(s_pop(&mut h[0], 0, &l[0]))
                    }
                    fn peek(&mut self) -> PathData {
                        s_peek(&self.p[0], 0, &self.p[1])
                    }
                    fn push(&mut self, e: PathData) {
                        let (h, l) = self.p.split_at_mut(1);
                        s_push(&mut h[0], 0, &mut l[0], e, self.log, "p0+0x0");
                    }
                    fn clear(&mut self) {
                        wr32(&mut self.p[0], 8, 0);
                    }
                    fn depth(&self) -> i32 {
                        rd32(&self.p[0], 8)
                    }
                    fn domain_below_2(&self) -> bool {
                        rd32(&self.p[6], 0x218) < 2
                    }
                    fn can_transport(&mut self) -> bool {
                        self.log.push("s0:p5+0x0".to_string());
                        self.a[6] != 0
                    }
                    fn tregion(&mut self, cx4: i32, cy4: i32) -> i32 {
                        // The goal's pair first, then the start's: two different
                        // locals.
                        if self.tregions.is_multiple_of(2) {
                            self.log.push("s1:c1+0x0,sp-0x48,sp-0x44".to_string());
                        } else {
                            self.log.push("s1:c1+0x0,sp-0x50,sp-0x4c".to_string());
                        }
                        let v = self.a[16 + self.tregions.min(23)];
                        self.tregions += 1;
                        self.tiles_x.push(hex(cx4));
                        self.tiles_y.push(hex(cy4));
                        v
                    }
                    fn valid_ucoord(&mut self, p: Pos, metric: i32) -> bool {
                        self.log
                            .push(format!("s2:c4+0x0,{},{},{}", p.x, p.y, metric));
                        let v = self.a[40 + self.valids.min(11)];
                        self.valids += 1;
                        v != 0
                    }
                    fn search(&mut self, anti: i32) -> i32 {
                        wr32(&mut self.c[4], 0x68, 0);
                        wr32(&mut self.c[4], 0x6c, 0);
                        wr32(&mut self.c[4], 0x64, 1);
                        self.log.push(format!("s3:c4+0x0,p0+0x0,48,{anti}"));
                        wr32(&mut self.c[4], 0x74, 0);
                        wr32(&mut self.c[4], 0x64, 0);
                        self.a[9]
                    }
                    fn suspended(&self, _r: i32) -> bool {
                        rd32(&self.c[4], 0x84) != 0
                    }
                    fn drop_unfinal_top(&mut self) {
                        let len = rd32(&self.p[0], 8);
                        let k = if len >= 1 { (len - 1) as usize * 16 } else { 0 };
                        if rd32(&self.p[1], k + 12) & 1 == 0 {
                            wr32(&mut self.p[0], 8, len.max(1) - 1);
                        }
                    }
                    fn order_is_move(&mut self) -> bool {
                        self.log.push("s5:p5+0x0".to_string());
                        self.log.push("s8:p8+0x0".to_string());
                        self.a[14] != 0
                    }
                    fn move_retry(&mut self) -> i32 {
                        self.log.push("s5:p5+0x0".to_string());
                        self.log.push("s9:p8+0x0".to_string());
                        rd32(&self.p[9], 0x1c)
                    }
                    fn kill_current_order(&mut self) {
                        self.log.push("s6:p5+0x0,0".to_string());
                    }
                    fn kill_lists(&mut self) {
                        self.log.push("s4:c4+0x0".to_string());
                    }
                    fn unit_pos(&self) -> Pos {
                        Pos::new(
                            rd32(&self.p[3], 0x10) ^ 0x63637,
                            rd32(&self.p[3], 0x14) ^ 0x63637,
                        )
                    }
                    fn keep_begin(&mut self) {
                        wr32(&mut self.c[6], 0xc, 0);
                    }
                    fn keep_push(&mut self, e: PathData) {
                        s_push(&mut self.c[6], 4, &mut self.p[7], e, self.log, "c6+0x4");
                    }
                    fn keep_pop(&mut self) -> Option<PathData> {
                        if rd32(&self.c[6], 0xc) == 0 {
                            return None;
                        }
                        Some(s_pop(&mut self.c[6], 4, &self.p[7]))
                    }
                }
                assert_eq!(a.len(), 52, "arguments and answers");
                let mut v = V {
                    a,
                    c,
                    p,
                    log: &mut log,
                    tiles_x: Vec::new(),
                    tiles_y: Vec::new(),
                    tregions: 0,
                    valids: 0,
                };
                let r = find_upath_steps(&mut v, Pos::new(a[1], a[2]), a[5]);
                let col = |w: &Vec<String>| {
                    if w.is_empty() {
                        "-".to_string()
                    } else {
                        w.join("/")
                    }
                };
                let (x, y) = (col(&v.tiles_x), col(&v.tiles_y));
                // No call recorded at all is a `-` column of its own.
                if v.log.is_empty() {
                    v.log.push("-".to_string());
                }
                v.log.push(x);
                v.log.push(y);
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "find_upath — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_upath — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_upath — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_upath — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "find_upath — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("PathFinder::find_upath@00682f30: {n} rows agree");
}
