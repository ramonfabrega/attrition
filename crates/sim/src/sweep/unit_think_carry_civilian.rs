//! `Unit::think_carry_civilian@005f37c0` under the emulator against the port (item 3346; tools/sweep_gen.py, obj with windows).

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

use crate::carry_civ::{CarryCivView, think_carry_civilian_steps};
use crate::movement::Angle;
use crate::world::{Cell, Pos};

/// The windows of one row as the port's view: `m` the boat, `c` the cells
/// (`c0` the game, `c1` the world, `c2` the division table, `c3` the
/// regions, `c4`/`c5` the step tables), `p0` the region array, `p1` the
/// world's cell array, `p2..p5` the danger tables, `p(5+slot)` a slot's
/// coordinates, `p70` the order the factory answers and `p71` its vbtable.
struct RowView<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    goes: usize,
    coasts: usize,
    hits: usize,
    raw_military: Option<i64>,
}

/// Where the sequences start in `a`: the workers argument, thirteen stub
/// answers, then the three `--seq`s of 64, 64 and 400.
const SEQ_GO: usize = 14;
const SEQ_COAST: usize = 78;
const SEQ_HERE: usize = 142;

impl RowView<'_> {
    fn region(slot: i32) -> String {
        format!("p0+{:#x}", 0x88 * slot)
    }
}

impl CarryCivView for RowView<'_> {
    fn owner(&self) -> i32 {
        i32::from(self.m[9])
    }
    fn index(&self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
    }
    fn frame(&self) -> i32 {
        rd(&self.c[0], 0x550)
    }
    fn pos(&self) -> Pos {
        Pos::new(rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
    }
    fn my_region(&mut self) -> i32 {
        self.a[1]
    }
    fn size(&self, slot: i32) -> i32 {
        rd(
            &self.p[0],
            usize::try_from(0x88 * slot + 0x14).expect("an offset"),
        )
    }
    fn go_here(&mut self, slot: i32) -> i32 {
        self.log
            .push(format!("s1:{},{}", Self::region(slot), self.owner()));
        let v = self.a[SEQ_GO + self.goes.min(63)];
        self.goes += 1;
        v
    }
    fn is_coast(&mut self, slot: i32, my: i32) -> bool {
        self.log.push(format!("s2:{},{my}", Self::region(slot)));
        let v = self.a[SEQ_COAST + self.coasts.min(63)];
        self.coasts += 1;
        v != 0
    }
    fn scouted(&self, slot: i32) -> bool {
        let who = usize::try_from(self.owner()).expect("an owner");
        let at = usize::try_from(0x88 * slot + 0x40).expect("an offset") + (who >> 3);
        self.p[0][at] & (1 << (who & 7)) != 0
    }
    fn coord(&mut self, slot: i32, i: i32) -> Cell {
        let w = &self.p[usize::try_from(5 + slot).expect("a window")];
        let o = 8 * usize::try_from(i).expect("a coordinate index");
        Cell::new(rd(w, o), rd(w, o + 4))
    }
    fn coast_here(&mut self, slot: i32, my: i32, c: Cell) -> i32 {
        self.log
            .push(format!("s3:{},{my},{},{}", Self::region(slot), c.x, c.y));
        let v = self.a[SEQ_HERE + self.hits.min(399)];
        self.hits += 1;
        v
    }
    fn danger(&self, c: Cell) -> i32 {
        let stride = rd(&self.c[1], 0x24);
        let who = usize::try_from(self.owner()).expect("an owner");
        let at = usize::try_from((c.y >> 1) * stride + (c.x >> 1)).expect("an index") * 4;
        rd(&self.p[2 + who], at)
    }
    fn step(&self, d: i32) -> (i32, i32) {
        let o = 4 * usize::try_from(d).expect("a direction");
        (rd(&self.c[4], o), rd(&self.c[5], o))
    }
    fn cell_slot(&self, c: Cell) -> Option<i32> {
        let (w, h) = (rd(&self.c[1], 0), rd(&self.c[1], 4));
        if c.x < 0 || c.y < 0 || c.x >= w || c.y >= h {
            return None;
        }
        let off = usize::try_from(w * c.y + c.x).expect("a cell") * 0x1c + 4;
        Some(i32::from(i16::from_le_bytes([
            self.p[1][off],
            self.p[1][off + 1],
        ])))
    }
    fn military(&mut self) -> bool {
        self.log.push("s12:m+0x0".to_string());
        self.raw_military = Some(i64::from(self.a[13]));
        self.a[13] != 0
    }
    fn order_move_to(&mut self, to: Pos, angle: Angle) {
        let f68 = rd(self.m, 0x68) & 0xfbff_ffffu32 as i32;
        wr(self.m, 0x68, f68);
        wr(self.m, 0xc0, 0);
        self.log.push("s4:m+0x0,0".to_string());
        self.log.push("s5:m+0x0".to_string());
        self.log.push("s6:m+0x0".to_string());
        self.log.push("s7:m+0x0,1".to_string());
        let disp = usize::try_from(rd(&self.p[71], 4)).expect("a displacement");
        let snap = |v: i32| {
            rd(
                &self.c[2],
                4 * usize::try_from(v >> 4).expect("a table index"),
            ) * 0x30
                + 0x18
        };
        let (xt, yt) = (snap(to.x), snap(to.y));
        let o = &mut self.p[70];
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
        self.log.push(format!("s9:m+0xcc,p70+{disp:#x}"));
        self.log.push("s6:m+0x0".to_string());
    }
    fn eject_all(&mut self) {
        self.log.push(format!(
            "s10:sp-0xa48,{},{},0,0",
            self.index(),
            self.owner()
        ));
        self.log.push("s11:sp-0xa48,0,-1,-1,-1".to_string());
    }
}

/// `Unit::think_carry_civilian@005f37c0`, run by `tools/emu/sweep_unit_think_carry_civilian.py` under unicorn
/// (tools/emu): 224 random bytes of object, 1 argument(s), `p` windows in order: p0..p71 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06188 points at 352 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; the global cell 0x00c061b8 points at 64 random bytes; 1156 random bytes at 0x00adcaf0 (each word in -1..1); 1156 random bytes at 0x00adc400 (each word in -1..1); callee 0x006b52e0 (2 args) stubbed; callee 0x006810f0 (1 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00681020 (3 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x00710b40 (4 args) stubbed; callee 0x005f3260 (0 args) stubbed; the word at `c3+0x10` points at 17408 random bytes less `0` (placed); the word at `c1+0x134` points at 2304 random bytes less `0` (placed); the word at `c1+0x13c` points at 256 random bytes less `0` (placed); the word at `c1+0x140` points at 256 random bytes less `0` (placed); the word at `c1+0x144` points at 256 random bytes less `0` (placed); the word at `c1+0x148` points at 256 random bytes less `0` (placed); the word at `p0+0x104` points at 12288 random bytes less `0` (placed); the word at `p0+0x18c` points at 12288 random bytes less `0` (placed); the word at `p0+0x214` points at 320 random bytes less `0` (placed); the word at `p0+0x29c` points at 320 random bytes less `0` (placed); the word at `p0+0x324` points at 320 random bytes less `0` (placed); the word at `p0+0x3ac` points at 320 random bytes less `0` (placed); the word at `p0+0x434` points at 320 random bytes less `0` (placed); the word at `p0+0x4bc` points at 320 random bytes less `0` (placed); the word at `p0+0x544` points at 320 random bytes less `0` (placed); the word at `p0+0x5cc` points at 320 random bytes less `0` (placed); the word at `p0+0x654` points at 320 random bytes less `0` (placed); the word at `p0+0x6dc` points at 320 random bytes less `0` (placed); the word at `p0+0x764` points at 320 random bytes less `0` (placed); the word at `p0+0x7ec` points at 320 random bytes less `0` (placed); the word at `p0+0x874` points at 320 random bytes less `0` (placed); the word at `p0+0x8fc` points at 320 random bytes less `0` (placed); the word at `p0+0x984` points at 320 random bytes less `0` (placed); the word at `p0+0xa0c` points at 320 random bytes less `0` (placed); the word at `p0+0xa94` points at 320 random bytes less `0` (placed); the word at `p0+0xb1c` points at 320 random bytes less `0` (placed); the word at `p0+0xba4` points at 320 random bytes less `0` (placed); the word at `p0+0xc2c` points at 320 random bytes less `0` (placed); the word at `p0+0xcb4` points at 320 random bytes less `0` (placed); the word at `p0+0xd3c` points at 320 random bytes less `0` (placed); the word at `p0+0xdc4` points at 320 random bytes less `0` (placed); the word at `p0+0xe4c` points at 320 random bytes less `0` (placed); the word at `p0+0xed4` points at 320 random bytes less `0` (placed); the word at `p0+0xf5c` points at 320 random bytes less `0` (placed); the word at `p0+0xfe4` points at 320 random bytes less `0` (placed); the word at `p0+0x106c` points at 320 random bytes less `0` (placed); the word at `p0+0x10f4` points at 320 random bytes less `0` (placed); the word at `p0+0x117c` points at 320 random bytes less `0` (placed); the word at `p0+0x1204` points at 320 random bytes less `0` (placed); the word at `p0+0x128c` points at 320 random bytes less `0` (placed); the word at `p0+0x1314` points at 320 random bytes less `0` (placed); the word at `p0+0x139c` points at 320 random bytes less `0` (placed); the word at `p0+0x1424` points at 320 random bytes less `0` (placed); the word at `p0+0x14ac` points at 320 random bytes less `0` (placed); the word at `p0+0x1534` points at 320 random bytes less `0` (placed); the word at `p0+0x15bc` points at 320 random bytes less `0` (placed); the word at `p0+0x1644` points at 320 random bytes less `0` (placed); the word at `p0+0x16cc` points at 320 random bytes less `0` (placed); the word at `p0+0x1754` points at 320 random bytes less `0` (placed); the word at `p0+0x17dc` points at 320 random bytes less `0` (placed); the word at `p0+0x1864` points at 320 random bytes less `0` (placed); the word at `p0+0x18ec` points at 320 random bytes less `0` (placed); the word at `p0+0x1974` points at 320 random bytes less `0` (placed); the word at `p0+0x19fc` points at 320 random bytes less `0` (placed); the word at `p0+0x1a84` points at 320 random bytes less `0` (placed); the word at `p0+0x1b0c` points at 320 random bytes less `0` (placed); the word at `p0+0x1b94` points at 320 random bytes less `0` (placed); the word at `p0+0x1c1c` points at 320 random bytes less `0` (placed); the word at `p0+0x1ca4` points at 320 random bytes less `0` (placed); the word at `p0+0x1d2c` points at 320 random bytes less `0` (placed); the word at `p0+0x1db4` points at 320 random bytes less `0` (placed); the word at `p0+0x1e3c` points at 320 random bytes less `0` (placed); the word at `p0+0x1ec4` points at 320 random bytes less `0` (placed); the word at `p0+0x1f4c` points at 320 random bytes less `0` (placed); the word at `p0+0x1fd4` points at 320 random bytes less `0` (placed); the word at `p0+0x205c` points at 320 random bytes less `0` (placed); the word at `p0+0x20e4` points at 320 random bytes less `0` (placed); the word at `p0+0x216c` points at 320 random bytes less `0` (placed); the word at `p0+0x21f4` points at 320 random bytes less `0` (placed); stub 7's answer points at 32 random bytes less `0` (placed); stub 8's answer points at 96 random bytes less `0` (placed); the word at `p70+0` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3, -1, 5, 7] (90%); stub 1 answered from [0, 0, 1, 2, 3, 1, 1, 2] (90%); stub 2 answered from [0, 1, 1, 1, 1] (90%); stub 3 answered from [0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8] (90%); stub 12 answered from [0, 1] (90%); stub 1 answers 64 values in call order, printed after the answers; stub 2 answers 64 values in call order, printed after the answers; stub 3 answers 400 values in call order, printed after the answers; stub 3 answers 0..8 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p69+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_carry_civilian.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let mut v = RowView {
                a: &a,
                m: &mut m[..],
                c: &c[..],
                p: &mut p[..],
                log: &mut log,
                goes: 0,
                coasts: 0,
                hits: 0,
                raw_military: None,
            };
            let ret = think_carry_civilian_steps(&mut v, a[0] != 0);
            v.raw_military.unwrap_or(i64::from(ret))
        };
        if log.is_empty() {
            log.push("-".to_string());
        }
        assert_eq!(got, want_eax, "think_carry_civilian — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "think_carry_civilian — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "think_carry_civilian — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_carry_civilian — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "think_carry_civilian — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::think_carry_civilian@005f37c0: {n} rows agree");
}
