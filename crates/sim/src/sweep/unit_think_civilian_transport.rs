//! `Unit::think_civilian_transport@005f40d0` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

use crate::civ_transport::{CivTransportView, think_civilian_transport_steps};
use crate::world::{Cell, Pos};

/// The windows of one row as the port's view: `m` the unit, `c` the cells
/// (`c0` the game, `c1` the world, `c2` the division table, `c3` the
/// regions, then five a leader `c(4+5·owner)`: flags, `city_num`,
/// `xport_peasants`, `reg_xport_peasants` from slot 0, `reg_cities` from two
/// bytes before slot 0), `p0..p3` the unit-type chain, `p4` the region
/// array, `p5` the world's cell array, `p6..p9` the danger tables and
/// `p(9+slot)` a land slot's coordinates.
struct RowView<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    goes: usize,
    coasts: usize,
    hits: usize,
    waters: usize,
}

/// Where the sequences start in `a`: the colonise argument, twelve stub
/// answers, then the four `--seq`s of 130, 400, 400 and 64.
const SEQ_GO: usize = 13;
const SEQ_COAST: usize = 143;
const SEQ_HERE: usize = 543;
const SEQ_WATER: usize = 943;

fn short(m: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([m[off], m[off + 1]]))
}

impl RowView<'_> {
    fn who(&self) -> usize {
        usize::from(self.m[9])
    }
    fn leader(&self, k: usize) -> usize {
        4 + 5 * self.who() + k
    }
    fn region(slot: i32) -> String {
        format!("p4+{:#x}", 0x88 * slot)
    }
    fn cell_at(&self, x: i32, y: i32) -> usize {
        usize::try_from(rd(&self.c[1], 0) * y + x).expect("a cell") * 0x1c
    }
}

impl CivTransportView for RowView<'_> {
    fn owner(&self) -> i32 {
        i32::from(self.m[9])
    }
    fn index(&self) -> i32 {
        short(self.m, 0xa)
    }
    fn frame(&self) -> i32 {
        rd(&self.c[0], 0x550)
    }
    fn pos(&self) -> Pos {
        Pos::new(rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
    }
    fn domain(&self) -> i32 {
        rd(&self.p[3], 0x218)
    }
    fn type_flags(&self) -> i32 {
        i32::from(self.p[3][0x2b4])
    }
    fn leader_flags(&self) -> i32 {
        rd(&self.c[self.leader(0)], 0)
    }
    fn transport_type(&mut self) -> i32 {
        self.log.push("s0:p1+0x0".to_string());
        self.a[1]
    }
    fn rec_flags68(&self) -> i32 {
        rd(&self.p[1], 0x68)
    }
    fn rec_flags6c(&self) -> i32 {
        rd(&self.p[1], 0x6c)
    }
    fn rec_type_flags(&self) -> i32 {
        i32::from(self.p[2][0x2b4])
    }
    fn can_transport(&mut self) -> bool {
        self.log.push("s1:m+0x0".to_string());
        self.a[2] != 0
    }
    fn exploring(&self) -> bool {
        rd(self.m, 0x68) & 0x100 != 0
    }
    fn think_scout(&mut self) -> i32 {
        self.log.push("s2:m+0x0,1".to_string());
        self.a[3]
    }
    fn my_region(&mut self) -> i32 {
        self.a[4]
    }
    fn xport_peasants(&self) -> i32 {
        rd(&self.c[self.leader(2)], 0)
    }
    fn city_num(&self) -> i32 {
        rd(&self.c[self.leader(1)], 0)
    }
    fn cell_owner(&self, cx: i32, cy: i32) -> i32 {
        i32::from(self.p[5][self.cell_at(cx, cy) + 0xf] as i8)
    }
    fn reg_cities(&self, slot: i32) -> i32 {
        short(
            &self.c[self.leader(4)],
            usize::try_from(2 + 2 * slot).expect("an offset"),
        )
    }
    fn size(&self, slot: i32) -> i32 {
        rd(
            &self.p[4],
            usize::try_from(0x88 * slot + 0x14).expect("an offset"),
        )
    }
    fn go_here(&mut self, slot: i32) -> i32 {
        self.log
            .push(format!("s4:{},{}", Self::region(slot), self.owner()));
        let v = self.a[SEQ_GO + self.goes.min(129)];
        self.goes += 1;
        v
    }
    fn is_coast(&mut self, sea: i32, other: i32) -> bool {
        self.log.push(format!("s5:{},{other}", Self::region(sea)));
        let v = self.a[SEQ_COAST + self.coasts.min(399)];
        self.coasts += 1;
        v != 0
    }
    fn scouted(&self, slot: i32) -> bool {
        let who = self.who();
        let at = usize::try_from(0x88 * slot + 0x40).expect("an offset") + (who >> 3);
        self.p[4][at] & (1 << (who & 7)) != 0
    }
    fn reg_xport(&self, slot: i32) -> i32 {
        short(
            &self.c[self.leader(3)],
            usize::try_from(2 * slot).expect("an offset"),
        )
    }
    fn coord(&mut self, slot: i32, i: i32) -> Cell {
        let w = &self.p[usize::try_from(9 + slot).expect("a window")];
        let o = 8 * usize::try_from(i).expect("a coordinate index");
        Cell::new(rd(w, o), rd(w, o + 4))
    }
    fn coast_here(&mut self, land: i32, sea: i32, c: Cell) -> i32 {
        self.log
            .push(format!("s6:{},{sea},{},{}", Self::region(land), c.x, c.y));
        let v = self.a[SEQ_HERE + self.hits.min(399)];
        self.hits += 1;
        v
    }
    fn num_waterhalf(&mut self, land: i32, _c: Cell) -> i32 {
        self.log
            .push(format!("s7:{},sp-0xa14,sp-0xa18", Self::region(land)));
        let v = self.a[SEQ_WATER + self.waters.min(63)];
        self.waters += 1;
        v
    }
    fn danger(&self, c: Cell) -> i32 {
        let stride = rd(&self.c[1], 0x24);
        let at = usize::try_from((c.y >> 1) * stride + (c.x >> 1)).expect("an index") * 4;
        rd(&self.p[6 + self.who()], at)
    }
    fn group_clear(&mut self) {
        self.log.push("s8:sp-0x9e8,-1".to_string());
    }
    fn group_add(&mut self) {
        self.log
            .push(format!("s9:sp-0x9e8,{},{},0,0", self.index(), self.owner()));
    }
    fn group_push(&mut self) -> i32 {
        self.log
            .push(format!("s10:sp-0x9e8,{},sp-0x9e8,1", self.owner()));
        self.a[11]
    }
    fn group_move_near(&mut self, handle: i32, x: i32, y: i32, kind: i32) {
        self.log.push(format!(
            "s11:{},{x},{y},0,2,0,0,{kind},0,-1,-1,0",
            handle.wrapping_mul(0x9d4)
        ));
    }
    fn cell_region(&self, c: Cell) -> i32 {
        short(&self.p[5], self.cell_at(c.x, c.y) + 4)
    }
    fn bump_xport(&mut self) {
        let k = self.leader(2);
        let v = rd(&self.c[k], 0).wrapping_add(1);
        wr(&mut self.c[k], 0, v);
    }
    fn bump_reg_xport(&mut self, slot: i32) {
        let k = self.leader(3);
        let at = usize::try_from(2 * slot).expect("an offset");
        let v = (short(&self.c[k], at) as i16).wrapping_add(1);
        self.c[k][at..at + 2].copy_from_slice(&v.to_le_bytes());
    }
}

/// `Unit::think_civilian_transport@005f40d0`, run by `tools/emu/sweep_unit_think_civilian_transport.py` under unicorn
/// (tools/emu): 224 random bytes of object, 1 argument(s), `p` windows in order: p0..p72 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06188 points at 352 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; the global cell 0x00c061b8 points at 64 random bytes; 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e3a788 (each word in 0..4); 4 random bytes at 0x00e3ad50 (each word in -2..2); 264 random bytes at 0x00e3b46e (each word in 0..1); 268 random bytes at 0x00e3b5ec (each word in -1..1); 4 random bytes at 0x00e4127c; 4 random bytes at 0x00e41674 (each word in 0..4); 4 random bytes at 0x00e41c3c (each word in -2..2); 264 random bytes at 0x00e4235a (each word in 0..1); 268 random bytes at 0x00e424d8 (each word in -1..1); 4 random bytes at 0x00e48168; 4 random bytes at 0x00e48560 (each word in 0..4); 4 random bytes at 0x00e48b28 (each word in -2..2); 264 random bytes at 0x00e49246 (each word in 0..1); 268 random bytes at 0x00e493c4 (each word in -1..1); 4 random bytes at 0x00e4f054; 4 random bytes at 0x00e4f44c (each word in 0..4); 4 random bytes at 0x00e4fa14 (each word in -2..2); 264 random bytes at 0x00e50132 (each word in 0..1); 268 random bytes at 0x00e502b0 (each word in -1..1); callee 0x0046f790 (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; callee 0x005f6010 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006810f0 (1 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x00681020 (3 args) stubbed; callee 0x006b4db0 (2 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070f9e0 (3 args) stubbed; callee 0x00704990 (11 args) stubbed; the word at `0xc0aec0+m[9]*0x1c` points at 4 random bytes less `max(i16(m,0xa),0)*4` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p1+0x18` points at 704 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `c3+0x10` points at 17408 random bytes less `0` (placed); the word at `c1+0x134` points at 3400 random bytes less `0` (placed); the word at `c1+0x13c` points at 256 random bytes less `0` (placed); the word at `c1+0x140` points at 256 random bytes less `0` (placed); the word at `c1+0x144` points at 256 random bytes less `0` (placed); the word at `c1+0x148` points at 256 random bytes less `0` (placed); the word at `p4+0x104` points at 16384 random bytes less `0` (placed); the word at `p4+0x18c` points at 16384 random bytes less `0` (placed); the word at `p4+0x214` points at 320 random bytes less `0` (placed); the word at `p4+0x29c` points at 320 random bytes less `0` (placed); the word at `p4+0x324` points at 320 random bytes less `0` (placed); the word at `p4+0x3ac` points at 320 random bytes less `0` (placed); the word at `p4+0x434` points at 320 random bytes less `0` (placed); the word at `p4+0x4bc` points at 320 random bytes less `0` (placed); the word at `p4+0x544` points at 320 random bytes less `0` (placed); the word at `p4+0x5cc` points at 320 random bytes less `0` (placed); the word at `p4+0x654` points at 320 random bytes less `0` (placed); the word at `p4+0x6dc` points at 320 random bytes less `0` (placed); the word at `p4+0x764` points at 320 random bytes less `0` (placed); the word at `p4+0x7ec` points at 320 random bytes less `0` (placed); the word at `p4+0x874` points at 320 random bytes less `0` (placed); the word at `p4+0x8fc` points at 320 random bytes less `0` (placed); the word at `p4+0x984` points at 320 random bytes less `0` (placed); the word at `p4+0xa0c` points at 320 random bytes less `0` (placed); the word at `p4+0xa94` points at 320 random bytes less `0` (placed); the word at `p4+0xb1c` points at 320 random bytes less `0` (placed); the word at `p4+0xba4` points at 320 random bytes less `0` (placed); the word at `p4+0xc2c` points at 320 random bytes less `0` (placed); the word at `p4+0xcb4` points at 320 random bytes less `0` (placed); the word at `p4+0xd3c` points at 320 random bytes less `0` (placed); the word at `p4+0xdc4` points at 320 random bytes less `0` (placed); the word at `p4+0xe4c` points at 320 random bytes less `0` (placed); the word at `p4+0xed4` points at 320 random bytes less `0` (placed); the word at `p4+0xf5c` points at 320 random bytes less `0` (placed); the word at `p4+0xfe4` points at 320 random bytes less `0` (placed); the word at `p4+0x106c` points at 320 random bytes less `0` (placed); the word at `p4+0x10f4` points at 320 random bytes less `0` (placed); the word at `p4+0x117c` points at 320 random bytes less `0` (placed); the word at `p4+0x1204` points at 320 random bytes less `0` (placed); the word at `p4+0x128c` points at 320 random bytes less `0` (placed); the word at `p4+0x1314` points at 320 random bytes less `0` (placed); the word at `p4+0x139c` points at 320 random bytes less `0` (placed); the word at `p4+0x1424` points at 320 random bytes less `0` (placed); the word at `p4+0x14ac` points at 320 random bytes less `0` (placed); the word at `p4+0x1534` points at 320 random bytes less `0` (placed); the word at `p4+0x15bc` points at 320 random bytes less `0` (placed); the word at `p4+0x1644` points at 320 random bytes less `0` (placed); the word at `p4+0x16cc` points at 320 random bytes less `0` (placed); the word at `p4+0x1754` points at 320 random bytes less `0` (placed); the word at `p4+0x17dc` points at 320 random bytes less `0` (placed); the word at `p4+0x1864` points at 320 random bytes less `0` (placed); the word at `p4+0x18ec` points at 320 random bytes less `0` (placed); the word at `p4+0x1974` points at 320 random bytes less `0` (placed); the word at `p4+0x19fc` points at 320 random bytes less `0` (placed); the word at `p4+0x1a84` points at 320 random bytes less `0` (placed); the word at `p4+0x1b0c` points at 320 random bytes less `0` (placed); the word at `p4+0x1b94` points at 320 random bytes less `0` (placed); the word at `p4+0x1c1c` points at 320 random bytes less `0` (placed); the word at `p4+0x1ca4` points at 320 random bytes less `0` (placed); the word at `p4+0x1d2c` points at 320 random bytes less `0` (placed); the word at `p4+0x1db4` points at 320 random bytes less `0` (placed); the word at `p4+0x1e3c` points at 320 random bytes less `0` (placed); the word at `p4+0x1ec4` points at 320 random bytes less `0` (placed); the word at `p4+0x1f4c` points at 320 random bytes less `0` (placed); the word at `p4+0x1fd4` points at 320 random bytes less `0` (placed); the word at `p4+0x205c` points at 320 random bytes less `0` (placed); the word at `p4+0x20e4` points at 320 random bytes less `0` (placed); the word at `p4+0x216c` points at 320 random bytes less `0` (placed); the word at `p4+0x21f4` points at 320 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3] (90%); stub 1 answered from [0, 1, 1, 1] (90%); stub 5 answered from [0, 0, 1] (90%); stub 7 answered from [0, 0, 1] (90%); stub 4 answers 130 values in call order, printed after the answers; stub 5 answers 400 values in call order, printed after the answers; stub 6 answers 400 values in call order, printed after the answers; stub 7 answers 64 values in call order, printed after the answers; stub 3 answers -1..100 every row; stub 4 answers 0..3 every row; stub 6 answers 0..8 every row; stub 10 answers 0..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_civilian_transport.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(24);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let mut v = RowView {
                a: &a,
                m: &mut m[..],
                c: &mut c[..],
                p: &mut p[..],
                log: &mut log,
                goes: 0,
                coasts: 0,
                hits: 0,
                waters: 0,
            };
            i64::from(think_civilian_transport_steps(&mut v, a[0] != 0))
        };
        if log.is_empty() {
            log.push("-".to_string());
        }
        assert_eq!(
            got, want_eax,
            "think_civilian_transport — eax, row `{line}`"
        );
        assert_eq!(
            log, want_log,
            "think_civilian_transport — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "think_civilian_transport — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "think_civilian_transport — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[25 + i],
                "think_civilian_transport — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Unit::think_civilian_transport@005f40d0: {n} rows agree");
}
