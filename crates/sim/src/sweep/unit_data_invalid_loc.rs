//! `UnitData::invalid_loc@00607c30` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::invalid_loc@00607c30`, run by `tools/emu/sweep_obj.py` on its row `unit_data_invalid_loc` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 8 argument(s), the global cell 0x00c061d0 points at 512 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; 4 random bytes at 0x00c0ad70; callee 0x006b53f0 (3 args) stubbed; callee 0x0046f8c0 (2 args) stubbed; callee 0x0046f960 (0 args) stubbed; callee 0x0046f880 (2 args) stubbed; callee 0x00659180 (7 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `m+0xb8` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 512 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 2 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_invalid_loc") else {
        return;
    };
    let obj = super::Obj {
        name: "invalid_loc",
        full: "UnitData::invalid_loc@00607c30",
        cells: 3,
        calls: true,
        eax: true,
        min_rows: 1500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Windows: `m` the unit; `c[0]` the world record (`+0x0` the cell
        // row width, `+0x18`/`+0x1c` the tile bounds), `c[1]` the
        // `div_3_table`, `c[2]` `objects+0x200` (`find_who`); `p[0..2]`
        // the leaders' rows (flags at `+0`), `p[2]` the unit's type,
        // `p[3]` its path list, `p[4]` the cell table (stride `0x1c`),
        // `p[5]` the tile table (words). Arguments: `a[0..8]` the
        // original's eight; `a[8..15]` the seven stubs' answers in the
        // order they are named; `a[15..19]` the `was_seen` sequence;
        // `a[19]` 1 when the unit's slot `0xb8` is `ObjectData::is`.
        struct Env<'a> {
            a: &'a [i32],
            m: &'a [u8],
            c: &'a [Vec<u8>],
            p: &'a [Vec<u8>],
            x: i32,
            y: i32,
            calls: Vec<String>,
            dx: [Vec<String>; 4],
            seen_n: usize,
        }
        fn hx(v: i32) -> String {
            v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
        }
        impl Env<'_> {
            fn w(&self) -> i32 {
                rd(&self.c[0], 0x18)
            }
            fn tile_at(&self, idx: i32) -> u16 {
                let o = idx as usize * 2;
                u16::from_le_bytes([self.p[5][o], self.p[5][o + 1]])
            }
        }
        impl crate::path::InvalidLocEnv for Env<'_> {
            fn was_seen(&mut self, fx: i32, fy: i32) -> bool {
                const NAMES: [(&str, &str); 4] = [
                    ("0x38", "0x34"),
                    ("0x3c", "0x14"),
                    ("0x40", "0x14"),
                    ("0x40", "0x14"),
                ];
                let k = self.seen_n.min(3);
                let (nx, ny) = NAMES[k];
                let who = i32::from(self.m[9]);
                self.calls.push(format!("s0:c0+0x0,sp-{nx},sp-{ny},{who}"));
                self.dx[0].push(hx(fx));
                self.dx[1].push(hx(fy));
                let ans = self.a[15 + k];
                self.seen_n += 1;
                ans != 0
            }
            fn tile_mask(&mut self) -> u16 {
                self.tile_at(self.w() * self.y + self.x)
            }
            fn cell_flags(&mut self) -> u16 {
                let idx = rd(&self.c[0], 0) * (self.y >> 2) + (self.x >> 2);
                let o = idx as usize * 0x1c;
                u16::from_le_bytes([self.p[4][o], self.p[4][o + 1]])
            }
            fn own_mask(&mut self) -> u16 {
                const KEY: i32 = 0x63637;
                let t = |off: usize| {
                    let i = (rd(self.m, off) ^ KEY) >> 6;
                    rd(&self.c[1], i as usize * 4)
                };
                let (col, row) = (t(0x10), t(0x14));
                self.tile_at(row * self.w() + col)
            }
            fn is_cliff(&mut self) -> bool {
                let m = i32::from(self.tile_mask());
                self.calls
                    .push(format!("s1:{},sp-0x20,sp-0x1c", (m & 0xff00) | (m & 3)));
                self.dx[2].push(hx(self.x));
                self.dx[3].push(hx(self.y));
                self.a[9] != 0
            }
            fn can_transport(&mut self) -> bool {
                self.calls.push("s2:m+0x0".into());
                self.a[10] != 0
            }
            fn is_carrier(&mut self) -> bool {
                if self.a[19] != 0 {
                    self.calls.push("s6:p2+0x0,351,1".into());
                    self.a[14] != 0
                } else {
                    self.calls.push("s5:m+0x0,351,1".into());
                    self.a[13] != 0
                }
            }
            fn is_built(&mut self) -> bool {
                self.calls.push("s3:c0+0x0,sp-0x20,sp-0x1c".into());
                self.a[11] != 0
            }
            fn building_who(&mut self) -> i32 {
                self.calls.push(format!(
                    "s4:12626800,{},{},c0+0x0,{},0,0,c0+0x0",
                    self.x,
                    self.y,
                    i32::from(self.m[9])
                ));
                rd(&self.c[2], 0)
            }
        }
        let owner = i32::from(m[9]);
        let path_len = rd(m, 0xc0);
        let inp = crate::path::InvalidLocIn {
            x: a[0],
            y: a[1],
            width: rd(&c[0], 0x18),
            height: rd(&c[0], 0x1c),
            owner,
            human: p[owner as usize][0] & 4 != 0,
            path_top_transport: path_len != 0 && p[3][(16 * path_len - 4) as usize] & 4 != 0,
            domain: rd(&p[2], 0x218),
            attack_nonzero: rd(&p[2], 0x1e8) != 0,
            forest_walker: rd(m, 0x6c) & 0x4000 != 0,
            auto_transport: rd(m, 0x68) & 0x800000 != 0,
            ignore_buildings: a[2] != 0,
            fog_relax: a[3] != 0,
            enemy_builds_only: a[4] != 0,
            transport_a: a[5] != 0,
            transport_b: a[6] != 0,
        };
        let mut env = Env {
            a,
            m: &*m,
            c: &*c,
            p: &*p,
            x: a[0],
            y: a[1],
            calls: Vec::new(),
            dx: Default::default(),
            seen_n: 0,
        };
        let r = crate::path::invalid_loc_steps(&inp, &mut env);
        if env.calls.is_empty() {
            log.push("-".into());
        }
        log.extend(env.calls.iter().cloned());
        for d in &env.dx {
            log.push(if d.is_empty() {
                "-".into()
            } else {
                d.join("/")
            });
        }
        i64::from(r)
    });
}
