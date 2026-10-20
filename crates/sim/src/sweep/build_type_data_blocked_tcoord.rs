//! `BuildTypeData::blocked_tcoord@00636db0` under the emulator against the port (item 3346; tools/sweep_gen.py, obj with windows).

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

use crate::build::{Ident, flags};
use crate::tcoord::{TcoordView, blocked_tcoord_steps};

/// The type index the original's `is(code, 0)` is asked with, per ident.
const fn code(id: Ident) -> i32 {
    match id {
        Ident::Dock => 0x1b0,
        Ident::OilWell => 0x1a5,
        Ident::OilPlatform => 0x1a6,
        Ident::Village => 0x19e,
        Ident::Fort => 0x1bb,
        _ => -1,
    }
}

fn i16_at(w: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
}

fn hex4(v: i32) -> String {
    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

/// The emulated original's row, answered to [`blocked_tcoord_steps`] call by
/// call. `a[0..4]` the arguments (x, y, who, exclude), `a[4..22]` the stubs'
/// single answers (unread), then each stub's `--seq` run: `is` (0) `a[22..30]`,
/// `is_city` (1) `a[30]`, `is_fort` (2) `a[31..34]`, gathers (3) `a[34]`, flat
/// (4) `a[35]`, `was_seen` (5) `a[36]`, `was_really_seen` (6) `a[37]`,
/// `can_transport` (7) `a[38]`, `has_tribe_bonus` (8) `a[39..41]`,
/// `find_building_placed_at` (9) `a[41]`, `get_tregion` (10) `a[42..47]`,
/// `get_good` (15) `a[47]`, `get_land` (16) `a[48]`, `get_amount` (17)
/// `a[49]`, then the four slot flags `a[50..54]` (1: the image function).
struct Emu<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a [Vec<u8>],
    p: &'a [Vec<u8>],
    /// Calls made so far, per stub.
    n: [usize; 18],
    /// The `--deref` columns, per call: `was_seen`'s two, `was_really_seen`'s,
    /// `get_tregion`'s and `get_land`'s.
    der: [Vec<String>; 8],
    /// Set by `unbuilt_tile`: the next `get_tregion` is that list's, whose
    /// locals sit elsewhere in the frame (city, fort).
    site: Option<bool>,
    log: &'a mut Vec<String>,
}

impl Emu<'_> {
    fn ans(&mut self, stub: usize, base: usize, len: usize) -> i32 {
        let i = self.n[stub].min(len - 1);
        self.n[stub] += 1;
        self.a[base + i]
    }
    fn deref(&mut self, col: usize, x: i32, y: i32) {
        self.der[col].push(hex4(x));
        self.der[col + 1].push(hex4(y));
    }
    fn finish(&mut self) {
        // `--record-order` prints `-` for a row that called nothing.
        if self.log.is_empty() {
            self.log.push("-".to_string());
        }
        for d in &self.der {
            self.log.push(if d.is_empty() {
                "-".to_string()
            } else {
                d.join("/")
            });
        }
    }
    fn wdata(&self, x: i32, y: i32) -> usize {
        (((y >> 2) * rd(&self.c[0], 0) + (x >> 2)) * 0x1c) as usize
    }
    fn tdata(&self, x: i32, y: i32) -> usize {
        ((rd(&self.c[0], 0x18) * y + x) * 2) as usize
    }
    fn list_win(&self, fort: bool, who: i32) -> usize {
        23 + who as usize + if fort { 4 } else { 0 }
    }
    fn count_win(&self, fort: bool, who: i32) -> usize {
        4 + 2 * who as usize + if fort { 8 } else { 0 }
    }
}

impl TcoordView for Emu<'_> {
    fn tile_dims(&self) -> (i32, i32) {
        (rd(&self.c[0], 0x18), rd(&self.c[0], 0x1c))
    }
    fn sem8(&self) -> bool {
        self.c[1][0x821] & 8 != 0
    }
    fn was_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool {
        self.log.push(format!("s5:c0+0x0,sp-0x24,sp-0x1c,{who}"));
        self.deref(0, hx, hy);
        self.ans(5, 36, 1) != 0
    }
    fn was_really_seen(&mut self, hx: i32, hy: i32, who: i32) -> bool {
        self.log.push(format!("s6:c0+0x0,sp-0x1c,sp-0x24,{who}"));
        self.deref(2, hx, hy);
        self.ans(6, 37, 1) != 0
    }
    fn cell_flags(&self, x: i32, y: i32) -> u16 {
        let o = self.wdata(x, y);
        u16::from_le_bytes([self.p[20][o], self.p[20][o + 1]])
    }
    fn tile_mask(&self, x: i32, y: i32) -> u16 {
        let o = self.tdata(x, y);
        u16::from_le_bytes([self.p[21][o], self.p[21][o + 1]])
    }
    fn cell_owner(&self, x: i32, y: i32) -> i32 {
        i32::from(self.p[20][self.wdata(x, y) + 0xf] as i8)
    }
    fn this_is(&mut self, id: Ident) -> bool {
        self.log.push(format!("s0:m+0x0,{},0", code(id)));
        self.ans(0, 22, 8) != 0
    }
    fn domain(&self) -> i32 {
        rd(self.m, 0x218)
    }
    fn can_transport(&mut self, who: i32) -> i32 {
        self.log.push(format!("s7:r{who}+0x0"));
        self.ans(7, 38, 1)
    }
    fn placed_at(&mut self, x: i32, y: i32, who: i32, exclude: i32) -> i32 {
        self.log
            .push(format!("s9:12626800,{x},{y},{who},{exclude},{who}"));
        self.ans(9, 41, 1)
    }
    fn flags(&self) -> u32 {
        rd(self.m, 0x2c0) as u32
    }
    fn is_city(&mut self) -> bool {
        if self.a[50] != 0 {
            return self.this_is(Ident::Village);
        }
        self.log.push("s1:m+0x0".to_string());
        self.ans(1, 30, 1) != 0
    }
    fn is_fort(&mut self) -> bool {
        if self.a[51] != 0 {
            return self.this_is(Ident::Fort);
        }
        self.log.push("s2:m+0x0".to_string());
        self.ans(2, 31, 3) != 0
    }
    fn gathers(&mut self) -> bool {
        if self.a[52] != 0 {
            return self.flags() & flags::GATHER != 0;
        }
        self.log.push("s3:m+0x0".to_string());
        self.ans(3, 34, 1) != 0
    }
    fn flat(&mut self) -> bool {
        if self.a[53] != 0 {
            return self.flags() & flags::FLAT != 0;
        }
        self.log.push("s4:m+0x0".to_string());
        self.ans(4, 35, 1) != 0
    }
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
        self.log.push(format!("s8:r{who}+0x0,{bonus}"));
        self.ans(8, 39, 2) != 0
    }
    fn dutch_fort_placement(&self) -> i32 {
        rd(&self.c[2], 0x8a4)
    }
    fn tregion(&mut self, x: i32, y: i32) -> i32 {
        let args = match self.site.take() {
            None => "sp-0x10,sp-0xc",
            Some(false) => "sp-0x8,sp-0x24",
            Some(true) => "sp-0x24,sp-0x20",
        };
        self.log.push(format!("s10:c0+0x0,{args}"));
        self.deref(4, x, y);
        self.ans(10, 42, 5)
    }
    fn reg_cities(&self, who: i32, tr: i32) -> i32 {
        i16_at(&self.p[who as usize], (2 * tr) as usize)
    }
    fn reg_forts(&self, who: i32, tr: i32) -> i32 {
        i16_at(&self.p[who as usize], 0x80 + (2 * tr) as usize)
    }
    fn unbuilt_count(&self, fort: bool, who: i32) -> i32 {
        rd(&self.p[self.count_win(fort, who)], 0)
    }
    fn unbuilt_o(&self, fort: bool, who: i32, k: i32) -> i32 {
        i16_at(&self.p[self.list_win(fort, who)], 4 * k as usize)
    }
    fn unbuilt_tile(&mut self, fort: bool, who: i32, k: i32) -> (i32, i32) {
        let item = format!("p{}+{:#x}", self.list_win(fort, who), 4 * k);
        let (gy, gx) = if fort { (13, 14) } else { (11, 12) };
        let coord = |me: &mut Self, s: usize| {
            let out = match s {
                11 => "sp-0x28",
                12 => "sp-0x2c",
                13 => "sp-0x30",
                _ => "sp-0x34",
            };
            me.log.push(format!("s{s}:{item},{out}"));
            let w = 31 + 2 * (s - 11) + me.n[s].min(1);
            me.n[s] += 1;
            let v = rd(&me.p[w], 0);
            rd(&me.c[4], 4 * (v >> 6) as usize)
        };
        let y = coord(self, gy);
        let x = coord(self, gx);
        self.site = Some(fort);
        (x, y)
    }
    fn good(&mut self) -> i32 {
        self.log.push("s15:m+0x0".to_string());
        self.ans(15, 47, 1)
    }
    fn land(&mut self, x: i32, y: i32) -> i32 {
        self.log.push("s16:c0+0x0,sp-0x10,sp-0xc,1".to_string());
        self.deref(6, x, y);
        self.ans(16, 48, 1)
    }
    fn land_amount(&mut self, land: i32, good: i32) -> i32 {
        // The second slot is the caller's stale `ecx`: the tile index when
        // both slot tests were the image's inline flag reads, else `this`.
        let stale = if self.a[52] != 0 && self.a[53] != 0 {
            (rd(&self.c[0], 0x18) * self.a[1] + self.a[0]).to_string()
        } else {
            "m+0x0".to_string()
        };
        self.log
            .push(format!("s17:p22+{:#x},{good},{stale}", land * 0x138));
        self.ans(17, 49, 1)
    }
}

/// `BuildTypeData::blocked_tcoord@00636db0`, run by `tools/emu/sweep_build_type_data_blocked_tcoord.py` under unicorn
/// (tools/emu): 768 random bytes of object, 4 argument(s), `p` windows in order: p0..p3 sparse table 0 (4 rows × 1 slice(s), row-major), p4..p11 sparse table 1 (4 rows × 2 slice(s), row-major), p12..p19 sparse table 2 (4 rows × 2 slice(s), row-major), p20..p30 the place(s), in flag order, p31..p38 the stubs' answer window(s); the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00c061e4 points at 2240 random bytes; the global cell 0x00c061dc points at 32 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; vtable slot +0x60 (2 args) stubbed; vtable slot +0x64 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x006b54f0 (3 args) stubbed; callee 0x006e0c60 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00658c80 (5 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0073b780 (1 args) stubbed; callee 0x0073b740 (1 args) stubbed; callee 0x0073bbc0 (1 args) stubbed; callee 0x0073bb80 (1 args) stubbed; callee 0x0063bd50 (0 args) stubbed; callee 0x006b4c70 (3 args) stubbed; callee 0x0067e6d0 (2 args) stubbed; the word at `c0+0x134` points at 512 random bytes less `0` (placed); the word at `c0+0x138` points at 512 random bytes less `0` (placed); the word at `c3+0x10` points at 1248 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 16 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; slot +0xfc laid as 0x00472ba0 or its stub, a flag after the answers; slot +0x90 laid as 0x00472bb0 or its stub, a flag after the answers; slot +0x94 laid as 0x00472b20 or its stub, a flag after the answers; stub 0 answered from [0, 0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 5 answered from [0, 1, 1, 1, 1] (90%); stub 6 answered from [0, 1, 1] (90%); stub 7 answered from [0, 0, 1, 2, 3] (90%); stub 8 answered from [0, 0, 0, 1] (90%); stub 9 answered from [-1, -1, 0, 1, 2] (90%); stub 15 answered from [-1, 0, 1, 2, 3, 4, 5] (90%); stub 17 answered from [0, 0, 1, 5] (90%); stub 11 answers a window of its own to each call, in call order (8 bytes, ...); stub 12 answers a window of its own to each call, in call order (8 bytes, ...); stub 13 answers a window of its own to each call, in call order (8 bytes, ...); stub 14 answers a window of its own to each call, in call order (8 bytes, ...); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 1 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; stub 3 answers 1 values in call order, printed after the answers; stub 4 answers 1 values in call order, printed after the answers; stub 5 answers 1 values in call order, printed after the answers; stub 6 answers 1 values in call order, printed after the answers; stub 7 answers 1 values in call order, printed after the answers; stub 8 answers 2 values in call order, printed after the answers; stub 9 answers 1 values in call order, printed after the answers; stub 10 answers 5 values in call order, printed after the answers; stub 15 answers 1 values in call order, printed after the answers; stub 16 answers 1 values in call order, printed after the answers; stub 17 answers 1 values in call order, printed after the answers; stub 10 answers 0..7 every row; stub 16 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 5 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 5 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 16 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 16 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 4 rows 0x6eec apart at `0xc061e0`, slices +0x125e:0x100 (`p` windows after the pointed buffers, before the places); 4 rows 0x1c apart at `@0xe39a40`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); 4 rows 0x1c apart at `@0xe39b20`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_blocked_tcoord.py") else {
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
            let mut e = Emu {
                a,
                m,
                c,
                p,
                n: [0; 18],
                der: Default::default(),
                site: None,
                log: &mut log,
            };
            let r = blocked_tcoord_steps(&mut e, a[2], a[0], a[1], a[3]);
            e.finish();
            i64::from(r)
        };
        assert_eq!(got, want_eax, "blocked_tcoord — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "blocked_tcoord — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "blocked_tcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "blocked_tcoord — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "blocked_tcoord — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("BuildTypeData::blocked_tcoord@00636db0: {n} rows agree");
}
