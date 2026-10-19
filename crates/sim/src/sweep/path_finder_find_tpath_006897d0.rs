//! `PathFinder::find_tpath@006897d0` under the emulator against the port (item 3337; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::find_tpath@006897d0`, run by `tools/emu/sweep_path_finder_find_tpath_006897d0.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), `p` windows in order: p0 argument 0, p1..p7 the place(s), in flag order, p8..p9 the stubs' answer window(s); the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c0618c points at 112 random bytes; 152 random bytes at 0x00e85e40; 112 random bytes at 0x00c0aec0; argument 0 names 16 random bytes; callee 0x00687d60 (2 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046f960 (0 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x00683770 (3 args) stubbed; callee 0x00687ae0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `c2+0x14+0x1c*a[3]` points at 12 random bytes less `0` (placed); the word at `p2+4*a[4]` points at 32 random bytes less `0` (placed); the word at `c4+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p4+4*a[4]` points at 32 random bytes less `0` (placed); the word at `p5+0x18` points at 704 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 7 answered from [0, 1, 1, 1] (90%); stub 8 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1, 2, 3, 4, 5] (90%); stub 4 answers the address of -32 random bytes, a `p` window after the places; stub 9 answers the address of 88 random bytes, a `p` window after the places; stub 0 answers 30 values in call order, printed after the answers; stub 1 answers 60 values in call order, printed after the answers; stub 5 answers -1..4 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 3 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_find_tpath_006897d0.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**,
                // `crate::tpath::find_tpath_steps`, which `Sim::find_tpath` runs
                // over its `SimTpath`; here the view is the emulated original's
                // windows.
                //
                // `a`: the five stack arguments (the `Stack<PathData>*`, the
                // from-position `x`, `y`, the unit class and slot), the ten
                // stubs' single answers (`a[5..15]`: 2 `can_transport`, 5
                // `astar_path`, 7 and 8 the slots `+0x18` and `+0x14`; the
                // others unused), stub 0's thirty `valid_tcoord` answers
                // (`a[15..45]`), stub 1's sixty `get_tregion` answers
                // (`a[45..105]`), then the one null flag of `update_order`
                // (`a[105]`). Stubs: 0 `valid_tcoord`, 1 `get_tregion` (its
                // two arguments are locals; the tile pair is printed after the
                // record), 2 `UnitData::can_transport`, 3 `Stack<PathData>::push`
                // (its argument a local whose 16 bytes the row prints), 4
                // `Unit::update_order`, 5 `astar_path`, 6 `kill_lists`, 7/8/9 the
                // slots `+0x18`, `+0x14`, `+0x40`. Windows: `c0` `div_3_table`,
                // `c1` the world (`+0x18`/`+0x1c` its extent in tiles), `c2` the
                // objects singleton, `c3` the pathfinder (`+0x54` the unit,
                // `+0x58`/`+0x5c` the start tile, `+0x68`/`+0x6c` the tile
                // offset, `+0x74`, `+0x84` the `saving` flag held 0, `+0x94`),
                // `c4` the units singleton; `p0` the stack (`+0` list, `+8`
                // length), `p1` its list, `p2` the objects row, `p3` the
                // object (`+0` vtable, `+0x18` type), `p4` the units row, `p5`
                // the unit data (`+0x18` type), `p6` its type (`+0x2b4`), `p7`
                // the object's type (`+0x218` domain), `p8` the order
                // `update_order` answers, `p9` the `MoveOrder` its slot `+0x40`
                // answers (`+0x4c`/`+0x4e`).
                use crate::Pos;
                use crate::orders::PathData;
                use crate::tpath::{TpathView, find_tpath_steps};
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
                fn hex(w: [i32; 4]) -> String {
                    let mut s = String::new();
                    for v in w {
                        for b in v.to_le_bytes() {
                            s.push_str(&format!("{b:02x}"));
                        }
                    }
                    s
                }
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    pushed: Vec<String>,
                    tiles_x: Vec<String>,
                    tiles_y: Vec<String>,
                    tregions: usize,
                    valids: usize,
                }
                impl TpathView for V<'_> {
                    fn begin(&mut self, start: Pos) {
                        // The original also writes `pathfinder +0x54` (the unit's
                        // pointer, from the units row), `+0x68`/`+0x6c` and
                        // `+0x94` (zero): fields the crate holds no copy of, so
                        // the emulated windows carry them.
                        let unit = self.p[4][4 * self.a[4] as usize..][..4].to_vec();
                        self.c[3][0x54..0x58].copy_from_slice(&unit);
                        wr32(&mut self.c[3], 0x58, start.x);
                        wr32(&mut self.c[3], 0x5c, start.y);
                        wr32(&mut self.c[3], 0x68, 0);
                        wr32(&mut self.c[3], 0x6c, 0);
                        wr32(&mut self.c[3], 0x94, 0);
                    }
                    fn pop(&mut self) -> Option<PathData> {
                        let len = rd32(&self.p[0], 8);
                        if len < 1 {
                            return None;
                        }
                        let k = (len - 1) as usize * 16;
                        wr32(&mut self.p[0], 8, len - 1);
                        let l = &self.p[1];
                        Some(PathData {
                            to: Pos::new(rd32(l, k), rd32(l, k + 4)),
                            tolerance: rd32(l, k + 8),
                            flags: rd32(l, k + 12) as u8,
                        })
                    }
                    fn push(&mut self, e: PathData) {
                        // The third push of a call is the second local.
                        let local = if self.pushed.len() == 2 { 0x68 } else { 0x58 };
                        self.log.push(format!("s3:p0+0x0,sp-0x{local:x}"));
                        self.pushed
                            .push(hex([e.to.x, e.to.y, e.tolerance, i32::from(e.flags)]));
                    }
                    fn clear(&mut self) {
                        wr32(&mut self.p[0], 8, 0);
                    }
                    fn depth(&self) -> i32 {
                        rd32(&self.p[0], 8)
                    }
                    fn tile_in_bounds(&self, t: Pos) -> bool {
                        t.x >= 0
                            && t.y >= 0
                            && t.x < rd32(&self.c[1], 0x18)
                            && t.y < rd32(&self.c[1], 0x1c)
                    }
                    fn is_helicopter(&self) -> bool {
                        rd32(&self.p[6], 0x2b4) & 0x20 != 0
                    }
                    fn domain_below_2(&self) -> bool {
                        rd32(&self.p[7], 0x218) < 2
                    }
                    fn can_transport(&mut self) -> bool {
                        self.log.push("s2:p5+0x0".to_string());
                        self.a[5 + 2] != 0
                    }
                    fn tregion(&mut self, t: Pos) -> i32 {
                        // The goal's pair first, then the start's: two different
                        // locals.
                        if self.tregions.is_multiple_of(2) {
                            self.log.push("s1:c1+0x0,sp-0x30,sp-0x34".to_string());
                        } else {
                            self.log.push("s1:c1+0x0,sp-0x40,sp-0x3c".to_string());
                        }
                        let v = self.a[45 + self.tregions.min(59)];
                        self.tregions += 1;
                        self.tiles_x.push(hex([t.x, 0, 0, 0])[..8].to_string());
                        self.tiles_y.push(hex([t.y, 0, 0, 0])[..8].to_string());
                        v
                    }
                    fn valid_tcoord(&mut self, p: Pos) -> bool {
                        self.log.push(format!("s0:c3+0x0,{},{}", p.x, p.y));
                        let v = self.a[15 + self.valids.min(29)];
                        self.valids += 1;
                        v != 0
                    }
                    fn order_offset(&mut self) -> Option<Pos> {
                        self.log.push("s7:p3+0x0".to_string());
                        if self.a[5 + 7] == 0 {
                            return None;
                        }
                        self.log.push("s4:p5+0x0".to_string());
                        if self.a[105] != 0 {
                            return None;
                        }
                        self.log.push("s4:p5+0x0".to_string());
                        self.log.push("s8:p8+0x0".to_string());
                        if self.a[5 + 8] == 0 {
                            return None;
                        }
                        self.log.push("s4:p5+0x0".to_string());
                        self.log.push("s9:p8+0x0".to_string());
                        Some(Pos::new(rd16(&self.p[9], 0x4c), rd16(&self.p[9], 0x4e)))
                    }
                    fn set_offset(&mut self, off: Pos) {
                        wr32(&mut self.c[3], 0x68, off.x);
                        wr32(&mut self.c[3], 0x6c, off.y);
                    }
                    fn search(&mut self) -> i32 {
                        self.log.push("s5:c3+0x0,p0+0x0,192,0".to_string());
                        // `pathfinder +0x74` zeroed before `kill_lists`.
                        wr32(&mut self.c[3], 0x74, 0);
                        self.log.push("s6:c3+0x0".to_string());
                        self.a[5 + 5]
                    }
                    fn drop_unfinal_top(&mut self) {
                        let len = rd32(&self.p[0], 8);
                        let k = if len >= 1 { (len - 1) as usize * 16 } else { 0 };
                        if rd32(&self.p[1], k + 12) & 1 == 0 {
                            wr32(&mut self.p[0], 8, len.max(1) - 1);
                        }
                    }
                }
                let mut v = V {
                    a,
                    c,
                    p,
                    log: &mut log,
                    pushed: Vec::new(),
                    tiles_x: Vec::new(),
                    tiles_y: Vec::new(),
                    tregions: 0,
                    valids: 0,
                };
                let r = find_tpath_steps(&mut v, Pos::new(a[1], a[2]));
                let col = |w: &Vec<String>| {
                    if w.is_empty() {
                        "-".to_string()
                    } else {
                        w.join("/")
                    }
                };
                let (x, y, pushed) = (col(&v.tiles_x), col(&v.tiles_y), col(&v.pushed));
                // No call recorded at all is a `-` column of its own.
                if v.log.is_empty() {
                    v.log.push("-".to_string());
                }
                v.log.push(x);
                v.log.push(y);
                v.log.push(pushed);
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "find_tpath — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_tpath — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_tpath — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_tpath — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "find_tpath — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("PathFinder::find_tpath@006897d0: {n} rows agree");
}
