//! `Unit::think_carry_idle@005f2f30` under the emulator against the port (item 3411; tools/sweep_gen.py, obj with windows).

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

// hand: view
use crate::carry_idle::{CarryIdleView, think_carry_idle_steps};
use crate::movement::Angle;
use crate::world::Pos;

/// The position key: the stored words are xor this.
const KEY: i32 = 0x63637;

/// Where the stub answers start in `a`: `a[0..10]` the ten stubs' answers,
/// `a[10..12]` the two out values of `find_nearby_spot`, `a[12..20]` the
/// eight `is_coast` answers in call order, `a[20..28]` the eight
/// `get_building` answers.
const OUT: usize = 10;
const SEQ_COAST: usize = 12;
const SEQ_DOCK: usize = 20;

/// The windows of one row as the port's view: `m` the boat, `c0` the
/// world cell, `c1` the regions cell, `c2` the division table, `c3..c6` the
/// four owners' city arrays, `c7..c10` their city counts, `p0` the region
/// array, `p1 + 4 * owner + i` a city, `p17` the order factory, `p18` the
/// order it answers and `p19` its vbtable.
struct RowView<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    derefs: Vec<String>,
    coasts: usize,
    docks: usize,
}

impl RowView<'_> {
    fn owner(&self) -> usize {
        usize::from(self.m[9])
    }
    fn city_win(&self, i: i32) -> usize {
        1 + 4 * self.owner() + usize::try_from(i).expect("a city")
    }
    fn le(v: i32) -> String {
        v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl CarryIdleView for RowView<'_> {
    fn pos(&self) -> Pos {
        Pos::new(rd(self.m, 0x10) ^ KEY, rd(self.m, 0x14) ^ KEY)
    }
    fn div3(&mut self, i: i32) -> i32 {
        rd(&self.c[2], 4 * usize::try_from(i).expect("a table index"))
    }
    fn tregion(&mut self, tx: i32, ty: i32) -> i32 {
        self.derefs.push(Self::le(tx));
        self.derefs.push(Self::le(ty));
        self.a[0]
    }
    fn num_cities(&self) -> i32 {
        rd(&self.c[7 + self.owner()], 0)
    }
    fn city_open(&self, i: i32) -> bool {
        self.p[self.city_win(i)][4] & 1 != 0
    }
    fn city_region(&self, i: i32) -> i32 {
        let w = &self.p[self.city_win(i)];
        i32::from(i16::from_le_bytes([w[0xa], w[0xb]]))
    }
    fn is_coast(&mut self, my: i32, city_region: i32) -> bool {
        self.log
            .push(format!("s1:p0+{:#x},{city_region}", 0x88 * my));
        let v = self.a[SEQ_COAST + self.coasts];
        self.coasts += 1;
        v != 0
    }
    fn has_dock(&mut self, i: i32) -> bool {
        // The two stale stack words `get_building` is handed beside `DOCK`:
        // the return address of the `is_coast` call and its argument.
        self.log.push(format!(
            "s2:p{}+0x0,432,6238175,{}",
            self.city_win(i),
            self.city_region(i)
        ));
        let v = self.a[SEQ_DOCK + self.docks];
        self.docks += 1;
        v != -1
    }
    fn city_pos(&self, i: i32) -> Pos {
        let w = &self.p[self.city_win(i)];
        Pos::new(rd(w, 0xc), rd(w, 0x10))
    }
    fn nearby_spot(&mut self, i: i32) -> Option<Pos> {
        let c = self.city_pos(i);
        let index = i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]));
        self.log.push(format!(
            "s3:{},{},{},sp-0x14,sp-0x10,3840,-1,0,1431655765,3,{index},{},0,0,-1,0,-1",
            rd(self.m, 0x18),
            c.x,
            c.y,
            self.owner()
        ));
        (self.a[3] == 0).then(|| Pos::new(self.a[OUT], self.a[OUT + 1]))
    }
    fn order_move_to(&mut self, to: Pos, angle: Angle) {
        let f68 = rd(self.m, 0x68) & 0xfbff_ffffu32 as i32;
        wr(self.m, 0x68, f68);
        wr(self.m, 0xc0, 0);
        self.log.push("s4:m+0x0,0".to_string());
        self.log.push("s5:m+0x0".to_string());
        self.log.push("s6:m+0x0".to_string());
        self.log.push("s7:m+0x0,1".to_string());
        let disp = usize::try_from(rd(&self.p[19], 4)).expect("a displacement");
        let snap = |v: i32| {
            rd(
                &self.c[2],
                4 * usize::try_from(v >> 4).expect("a table index"),
            ) * 0x30
                + 0x18
        };
        let (xt, yt) = (snap(to.x), snap(to.y));
        let o = &mut self.p[18];
        wr(o, 4, xt);
        wr(o, 8, yt);
        wr(o, 0xc, angle.0);
        wr(o, 0x10, 0);
        wr(o, 0x2c, xt);
        wr(o, 0x30, yt);
        wr(o, 0x34, -1);
        wr(o, 0x38, -1);
        o[disp + 4] &= 0xda;
        o[0x4c..0x4e].copy_from_slice(&((xt % 0x300) as i16).to_le_bytes());
        o[0x4e..0x50].copy_from_slice(&((yt % 0x300) as i16).to_le_bytes());
        wr(o, 0x44, -1);
        wr(o, 0x48, -1);
        wr(o, 0x18, 0);
        wr(o, 0x1c, 0);
        wr(o, 0x24, 0);
        wr(o, 0x28, -1);
        self.log.push(format!("s9:m+0xcc,p18+{disp:#x}"));
        self.log.push("s6:m+0x0".to_string());
    }
}
// end hand

/// `Unit::think_carry_idle@005f2f30`, run by `tools/emu/sweep_unit_think_carry_idle.py` under unicorn
/// (tools/emu): 224 random bytes of object, 0 argument(s), `p` windows in order: p0..p19 the place(s), in flag order; the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00c061b8 points at 64 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; the global cell 0x00c09970 points at 32 random bytes; the global cell 0x00c0998c points at 32 random bytes; the global cell 0x00c099a8 points at 32 random bytes; the global cell 0x00c099c4 points at 32 random bytes; 4 random bytes at 0x00e3a798 (each word in 0..4); 4 random bytes at 0x00e41684 (each word in 0..4); 4 random bytes at 0x00e48570 (each word in 0..4); 4 random bytes at 0x00e4f45c (each word in 0..4); callee 0x006b52e0 (2 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00739480 (3 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `c1+0x10` points at 1088 random bytes less `0` (placed); the word at `c3+0` points at 32 random bytes less `0` (placed); the word at `c3+4` points at 32 random bytes less `0` (placed); the word at `c3+8` points at 32 random bytes less `0` (placed); the word at `c3+12` points at 32 random bytes less `0` (placed); the word at `c4+0` points at 32 random bytes less `0` (placed); the word at `c4+4` points at 32 random bytes less `0` (placed); the word at `c4+8` points at 32 random bytes less `0` (placed); the word at `c4+12` points at 32 random bytes less `0` (placed); the word at `c5+0` points at 32 random bytes less `0` (placed); the word at `c5+4` points at 32 random bytes less `0` (placed); the word at `c5+8` points at 32 random bytes less `0` (placed); the word at `c5+12` points at 32 random bytes less `0` (placed); the word at `c6+0` points at 32 random bytes less `0` (placed); the word at `c6+4` points at 32 random bytes less `0` (placed); the word at `c6+8` points at 32 random bytes less `0` (placed); the word at `c6+12` points at 32 random bytes less `0` (placed); stub 7's answer points at 32 random bytes less `0` (placed); stub 8's answer points at 96 random bytes less `0` (placed); the word at `p18+0` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 1, 1, 0] (90%); stub 2 answered from [-1, 0, 5, -1, 7] (90%); stub 3 answered from [0, 0, 0, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 0 answers 0..7 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 3 writes a value in [16, 128, 511, 576, 768, 1056, 1792, 4096, 6144] through its argument 2, printed after the answers (`o` in a place); stub 3 writes a value in [16, 128, 511, 576, 768, 1056, 1792, 4096, 6144] through its argument 3, printed after the answers (`o` in a place); the stubbed vtable laid at `p17+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_carry_idle.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(11);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let (ret, mut derefs) = {
                let mut v = RowView {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    derefs: Vec::new(),
                    coasts: 0,
                    docks: 0,
                };
                let ret = think_carry_idle_steps(&mut v);
                (ret, std::mem::take(&mut v.derefs))
            };
            if log.is_empty() {
                log.push("-".to_string());
            }
            log.append(&mut derefs);
            i64::from(ret)
        };
        assert_eq!(got, want_eax, "think_carry_idle — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "think_carry_idle — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_carry_idle — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_carry_idle — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[12 + i],
                "think_carry_idle — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::think_carry_idle@005f2f30: {n} rows agree");
}
