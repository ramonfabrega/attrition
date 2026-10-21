//! `Unit::think_carry_military@005f3260` under the emulator against the port (item 3411; tools/sweep_gen.py, obj with windows).

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

use crate::carry_mil::{CarryMilView, City, think_carry_military_steps};
use crate::movement::Angle;
use crate::world::{Cell, Pos};

fn rd16(m: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([m[off], m[off + 1]]))
}

fn us(v: i32) -> usize {
    usize::try_from(v).expect("a non-negative index")
}

/// The windows of one row as the port's view: `m` the boat; `c0` the world,
/// `c1` the division table, `c2` the regions' header, `c3` the groups, `c4`
/// the objects' header, `c5`/`c6` `move_x`/`move_y`, `c7`/`c8` the leaders'
/// first two rows' city count and region marks; `p1..` the units of table 0
/// (`p0` its array) and table 1 from `p6`, the armies `p11..`/`p18..`, the
/// cities `p25..`/`p33..`, the objects `p41..`/`p46..`, `p50` the regions,
/// `p51` the world's cells, `p52..` the regions' centres, `p59` the factory's
/// answer, `p60` the order and `p61` its vbtable. `a[9]` is the flag for the
/// vtable slot laid as `Unit::die`.
struct RowView<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
}

impl RowView<'_> {
    /// The unit-table row: the byte at `this+0x3e`, sign-extended.
    fn table(&self) -> i32 {
        i32::from(self.m[0x3e] as i8)
    }
}

impl CarryMilView for RowView<'_> {
    fn cargo_group(&mut self) -> i32 {
        let unit = &self.p[us(1 + 5 * self.table() + rd16(self.m, 0x28))];
        rd16(unit, 0x80)
    }
    fn group_army(&mut self, group: i32) -> i32 {
        rd(&self.c[3], us(group * 0x9d4 + 8))
    }
    fn army(&mut self, army: i32) -> (i32, i32, i32) {
        let r = &self.p[us(11 + 7 * self.table() + army)];
        (rd(r, 0x30), rd(r, 8), rd(r, 0x34))
    }
    fn cargo_slot(&mut self) -> i32 {
        rd16(self.m, 0x28)
    }
    fn eject(&mut self) {
        self.log.push("s0:m+0x0,0,-1,0,1".to_string());
        let s = (rd16(self.m, 0x28) as i16).wrapping_sub(1);
        self.m[0x28..0x2a].copy_from_slice(&s.to_le_bytes());
    }
    fn die_is_base(&mut self) -> bool {
        self.a[9] != 0
    }
    fn die_base(&mut self) {
        self.log.push("s2:m+0x0,0,-1,0".to_string());
    }
    fn die_virtual(&mut self) {
        self.log.push("s1:m+0x0,0,-1,0".to_string());
    }
    fn owner(&self) -> i32 {
        i32::from(self.m[9])
    }
    fn pos(&self) -> Pos {
        Pos::new(rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
    }
    fn city_count(&mut self, who: i32) -> i32 {
        rd(&self.c[us(7 + who)], 0)
    }
    fn city(&mut self, who: i32, i: i32) -> City {
        let r = &self.p[us(25 + 8 * who + i)];
        City {
            flags: u16::from_le_bytes([r[4], r[5]]),
            obj: rd16(r, 8),
            region: rd16(r, 0xa),
            x: rd(r, 0xc),
            y: rd(r, 0x10),
        }
    }
    fn div3(&mut self, i: i32) -> i32 {
        rd(&self.c[1], 4 * us(i))
    }
    fn step(&self, k: i32) -> (i32, i32) {
        let o = 4 * us(k);
        (rd(&self.c[5], o), rd(&self.c[6], o))
    }
    fn world_size(&mut self) -> (i32, i32) {
        (rd(&self.c[0], 0), rd(&self.c[0], 4))
    }
    fn cell_region(&mut self, c: Cell) -> i32 {
        let w = rd(&self.c[0], 0);
        rd16(&self.p[51], us(w * c.y + c.x) * 0x1c + 4)
    }
    fn object(&mut self, who: i32, index: i32) -> (bool, i32, i32) {
        let r = &self.p[us(41 + 5 * who + index)];
        (rd(r, 8) & 1 != 0, rd(r, 0x10), rd(r, 0x14))
    }
    fn region_centre(&mut self, region: i32) -> (i32, i32) {
        let r = &self.p[us(52 + region)];
        (rd(r, 0), rd(r, 4))
    }
    fn region_marked(&mut self, who: i32, region: i32) -> bool {
        self.c[us(7 + who)][0x660 + 2 * us(region)] & 4 != 0
    }
    fn order_move_to(&mut self, to: Pos, angle: Angle) {
        let f68 = rd(self.m, 0x68) & 0xfbff_ffffu32 as i32;
        wr(self.m, 0x68, f68);
        wr(self.m, 0xc0, 0);
        self.log.push("s3:m+0x0,0".to_string());
        self.log.push("s4:m+0x0".to_string());
        self.log.push("s5:m+0x0".to_string());
        self.log.push("s6:m+0x0,1".to_string());
        self.log.push("s7:p59+0x0".to_string());
        let disp = us(rd(&self.p[61], 4));
        let snap = |v: i32| rd(&self.c[1], 4 * us(v >> 4)) * 0x30 + 0x18;
        let (xt, yt) = (snap(to.x), snap(to.y));
        let o = &mut self.p[60];
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
        self.log.push(format!("s8:m+0xcc,p60+{disp:#x}"));
        self.log.push("s5:m+0x0".to_string());
    }
}

/// `Unit::think_carry_military@005f3260`, run by `tools/emu/sweep_unit_think_carry_military.py` under unicorn
/// (tools/emu): 208 random bytes of object, 0 argument(s), `p` windows in order: p0..p61 the place(s), in flag order; the global cell 0x00c06188 points at 352 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; the global cell 0x00c061b8 points at 64 random bytes; the global cell 0x00e85f20 points at 10064 random bytes; the global cell 0x00c0618c points at 256 random bytes; 1156 random bytes at 0x00adcaf0 (each word in -1..1); 1156 random bytes at 0x00adc400 (each word in -1..1); 1664 random bytes at 0x00e3a798 (each word in 0..7); 1664 random bytes at 0x00e41684 (each word in 0..7); callee 0x0064cd20 (4 args) stubbed; vtable slot +0x158 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 144 random bytes less `0` (placed); the word at `p0+4` points at 144 random bytes less `0` (placed); the word at `p0+8` points at 144 random bytes less `0` (placed); the word at `p0+12` points at 144 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 144 random bytes less `0` (placed); the word at `p5+4` points at 144 random bytes less `0` (placed); the word at `p5+8` points at 144 random bytes less `0` (placed); the word at `p5+12` points at 144 random bytes less `0` (placed); the word at `0xc09710` points at 24 random bytes less `0` (placed); the word at `p10+0` points at 56 random bytes less `0` (placed); the word at `p10+4` points at 56 random bytes less `0` (placed); the word at `p10+8` points at 56 random bytes less `0` (placed); the word at `p10+12` points at 56 random bytes less `0` (placed); the word at `p10+16` points at 56 random bytes less `0` (placed); the word at `p10+20` points at 56 random bytes less `0` (placed); the word at `0xc0972c` points at 24 random bytes less `0` (placed); the word at `p17+0` points at 56 random bytes less `0` (placed); the word at `p17+4` points at 56 random bytes less `0` (placed); the word at `p17+8` points at 56 random bytes less `0` (placed); the word at `p17+12` points at 56 random bytes less `0` (placed); the word at `p17+16` points at 56 random bytes less `0` (placed); the word at `p17+20` points at 56 random bytes less `0` (placed); the word at `0xc09970` points at 28 random bytes less `0` (placed); the word at `p24+0` points at 20 random bytes less `0` (placed); the word at `p24+4` points at 20 random bytes less `0` (placed); the word at `p24+8` points at 20 random bytes less `0` (placed); the word at `p24+12` points at 20 random bytes less `0` (placed); the word at `p24+16` points at 20 random bytes less `0` (placed); the word at `p24+20` points at 20 random bytes less `0` (placed); the word at `p24+24` points at 20 random bytes less `0` (placed); the word at `0xc0998c` points at 28 random bytes less `0` (placed); the word at `p32+0` points at 20 random bytes less `0` (placed); the word at `p32+4` points at 20 random bytes less `0` (placed); the word at `p32+8` points at 20 random bytes less `0` (placed); the word at `p32+12` points at 20 random bytes less `0` (placed); the word at `p32+16` points at 20 random bytes less `0` (placed); the word at `p32+20` points at 20 random bytes less `0` (placed); the word at `p32+24` points at 20 random bytes less `0` (placed); the word at `c4+0x14` points at 16 random bytes less `0` (placed); the word at `p40+0` points at 24 random bytes less `0` (placed); the word at `p40+4` points at 24 random bytes less `0` (placed); the word at `p40+8` points at 24 random bytes less `0` (placed); the word at `p40+12` points at 24 random bytes less `0` (placed); the word at `c4+0x30` points at 16 random bytes less `0` (placed); the word at `p45+0` points at 24 random bytes less `0` (placed); the word at `p45+4` points at 24 random bytes less `0` (placed); the word at `p45+8` points at 24 random bytes less `0` (placed); the word at `p45+12` points at 24 random bytes less `0` (placed); the word at `c2+0x10` points at 960 random bytes less `0` (placed); the word at `c0+0x134` points at 2304 random bytes less `0` (placed); the word at `p50+0x7c` points at 8 random bytes less `0` (placed); the word at `p50+0x104` points at 8 random bytes less `0` (placed); the word at `p50+0x18c` points at 8 random bytes less `0` (placed); the word at `p50+0x214` points at 8 random bytes less `0` (placed); the word at `p50+0x29c` points at 8 random bytes less `0` (placed); the word at `p50+0x324` points at 8 random bytes less `0` (placed); the word at `p50+0x3ac` points at 8 random bytes less `0` (placed); stub 6's answer points at 32 random bytes less `0` (placed); stub 7's answer points at 96 random bytes less `0` (placed); the word at `p60+0` points at 32 random bytes less `0` (placed); slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 lowers `m+0x28` (2 bytes) by one each call; the stubbed vtable laid at `p59+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_carry_military.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(9);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let mut v = RowView {
                a,
                m,
                c,
                p,
                log: &mut log,
            };
            let r = think_carry_military_steps(&mut v);
            if log.is_empty() {
                log.push("-".to_string());
            }
            i64::from(r)
        };
        assert_eq!(got, want_eax, "think_carry_military — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "think_carry_military — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "think_carry_military — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_carry_military — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[10 + i],
                "think_carry_military — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Unit::think_carry_military@005f3260: {n} rows agree");
}
