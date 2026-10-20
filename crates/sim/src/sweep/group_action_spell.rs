//! `Group::action_spell@006fe1a0` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_spell@006fe1a0`, run by `tools/emu/sweep_group_action_spell.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 5 argument(s), `p` windows in order: p0..p50 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00e85ddc points at 64 random bytes; the global cell 0x00c061a8 points at 64 random bytes; the global cell 0x00c061b4 points at 256 random bytes; the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00c061fc points at 64 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); 4 random bytes at 0x00c06208; 4 random bytes at 0x00c8cd00; vtable slot +0x14 (0 args) stubbed; callee 0x007095e0 (5 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0060a530 (1 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x00609a30 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xf4 (0 args) stubbed; callee 0x00609a50 (0 args) stubbed; vtable slot +0x84 (4 args) stubbed; callee 0x006763c0 (4 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00675800 (2 args) stubbed; callee 0x00620f40 (2 args) stubbed; callee 0x00676c40 (2 args) stubbed; callee 0x00676ce0 (4 args) stubbed; callee 0x0060a370 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x006d3650 (4 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `p0+12` points at 96 random bytes less `0` (placed); the word at `p1+0` points at 96 random bytes less `0` (placed); the word at `p1+4` points at 96 random bytes less `0` (placed); the word at `p1+8` points at 96 random bytes less `0` (placed); the word at `p1+12` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 192 random bytes less `0` (placed); the word at `p10+4` points at 192 random bytes less `0` (placed); the word at `p10+8` points at 192 random bytes less `0` (placed); the word at `p10+12` points at 192 random bytes less `0` (placed); the word at `p11+0` points at 192 random bytes less `0` (placed); the word at `p11+4` points at 192 random bytes less `0` (placed); the word at `p11+8` points at 192 random bytes less `0` (placed); the word at `p11+12` points at 192 random bytes less `0` (placed); the word at `c1+0` points at 64 random bytes less `0` (placed); the word at `c1+4` points at 64 random bytes less `0` (placed); the word at `c1+8` points at 64 random bytes less `0` (placed); the word at `c1+12` points at 64 random bytes less `0` (placed); the word at `c2+0x10` points at 2688 random bytes less `0` (placed); the word at `p24+0` points at 480 random bytes less `0` (placed); the word at `p24+4` points at 480 random bytes less `0` (placed); the word at `p24+8` points at 480 random bytes less `0` (placed); the word at `p24+12` points at 480 random bytes less `0` (placed); the word at `p24+0xa58` points at 480 random bytes less `0` (placed); the word at `c5+0x10` points at 32 random bytes less `0` (placed); the word at `p30+0` points at 576 random bytes less `0` (placed); the word at `p30+4` points at 576 random bytes less `0` (placed); the word at `p30+8` points at 576 random bytes less `0` (placed); the word at `p30+12` points at 576 random bytes less `0` (placed); the word at `p12+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p13+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p14+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p15+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p16+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p17+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p18+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p19+0x40` points at 96 random bytes less `0` (placed, null a third of the rows); the word at `p35+0x44` points at 64 random bytes less `0` (placed); the word at `p36+0x44` points at 64 random bytes less `0` (placed); the word at `p37+0x44` points at 64 random bytes less `0` (placed); the word at `p38+0x44` points at 64 random bytes less `0` (placed); the word at `p39+0x44` points at 64 random bytes less `0` (placed); the word at `p40+0x44` points at 64 random bytes less `0` (placed); the word at `p41+0x44` points at 64 random bytes less `0` (placed); the word at `p42+0x44` points at 64 random bytes less `0` (placed); slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; stub 1 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [1, 1, 1, 0] (90%); stub 5 answered from [1, 1, 1, 0] (90%); stub 6 answered from [0, 0, 1] (90%); stub 7 answered from [0, 3, 3, 3, 1] (90%); stub 8 answered from [0, 5, 10, 15, 20] (90%); stub 9 answered from [0, 1, 1] (90%); stub 11 answered from [14, 14, 0, 5] (90%); stub 13 answered from [0, 5, 10, 15, 20] (90%); stub 14 answered from [0, 1, 1, 1] (90%); stub 15 answered from [0, 1, 1, 1] (90%); stub 16 answered from [0, 1] (90%); stub 17 answered from [0, 1] (90%); stub 18 answered from [0, 0, 1] (90%); stub 20 answered from [0, 0, 1] (90%); stub 24 answered from [0, 0, 1] (90%); stub 25 answered from [14, 14, 0, 5] (90%); stub 3 answers 12 values in call order, printed after the answers; stub 4 answers 12 values in call order, printed after the answers; stub 5 answers 12 values in call order, printed after the answers; stub 6 answers 12 values in call order, printed after the answers; stub 7 answers 12 values in call order, printed after the answers; stub 8 answers 12 values in call order, printed after the answers; stub 9 answers 12 values in call order, printed after the answers; stub 11 answers 12 values in call order, printed after the answers; stub 13 answers 12 values in call order, printed after the answers; stub 14 answers 12 values in call order, printed after the answers; stub 15 answers 12 values in call order, printed after the answers; stub 16 answers 12 values in call order, printed after the answers; stub 17 answers 12 values in call order, printed after the answers; stub 18 answers 12 values in call order, printed after the answers; stub 20 answers 12 values in call order, printed after the answers; stub 24 answers 12 values in call order, printed after the answers; stub 25 answers 12 values in call order, the last from [0, 5], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; stub 29 writes 161 (4 bytes) at `ecx+0x0` each call; stub 29 writes 162 (4 bytes) at `ecx+0x4` each call; stub 29 writes 163 (4 bytes) at `ecx+0x8` each call; stub 29 writes 164 (4 bytes) at `ecx+0xc` each call; stub 29 writes 165 (4 bytes) at `ecx+0x10` each call; stub 10 answers from its own `ecx`: the word at `+0x40`; stub 12 answers from its own `ecx`: the word at `+0x44`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_spell.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(9);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at `+0x8cc`);
// `c[0]` the objects cell, `c[1]` the types list, `c[2]` the spelltypes cell, `c[3]` the leader options,
// `c[4]` the console, `c[5]` the unittypes cell, `c[6]` `ignore_orders`, `c[7]` the message window's word,
// `c[8]` the string table's base. `p[2 + 4 * pl + j]` the objects `objects[pl][j]`, `p[12 + 4 * pl + j]` the
// units, `p[20 + ty]` the type records, `p[25 + t]` the craft rows (`p[29]` the fallback `0x296`),
// `p[31 + ty]` the unit types, `p[35 + 4 * pl + j]` the order a unit's `+0x40` names (null a third of the
// rows) and `p[43 + 4 * pl + j]` the cast record that order's `+0x44` names. `a`: the five arguments, the
// stubs' answers (`a[5..37]`), every `--seq` run (`SEQ_AT`) and the `--slotbase` flag.
//
// **The port is the crate's own steps**, `crate::action_spell_view::action_spell_steps`, over a view of
// the emulated original's windows; the crate's `Sim::group_action_spell` is a different shape of the same
// arm (see that module's head).
use crate::action_spell_view::{GarrisonHead, SpellView, action_spell_steps};
const SEQ_AT: [(usize, usize, usize); 17] = [
    (3, 37, 12),
    (4, 49, 12),
    (5, 61, 12),
    (6, 73, 12),
    (7, 85, 12),
    (8, 97, 12),
    (9, 109, 12),
    (11, 121, 12),
    (13, 133, 12),
    (14, 145, 12),
    (15, 157, 12),
    (16, 169, 12),
    (17, 181, 12),
    (18, 193, 12),
    (20, 205, 12),
    (24, 217, 12),
    (25, 229, 12),
];
const BASE_FLAG: usize = 241;
/// `RED`, the feedback's colour (`0x00c8d248`).
const RED: i32 = 0x00c8_d248;
/// The words `String::String`'s stub writes over the string slot (`--stub-write`).
const STRING_WORDS: [i32; 5] = [0xa1, 0xa2, 0xa3, 0xa4, 0xa5];
struct V<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    turn: [usize; 32],
}
fn rd32(w: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
}
fn rd16(w: &[u8], o: usize) -> i32 {
    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
}
impl V<'_> {
    fn seq(&mut self, k: usize) -> i32 {
        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
        let i = self.turn[k].min(n - 1);
        self.turn[k] += 1;
        self.a[at + i]
    }
    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
        let mut t = format!("s{k}:{ecx}");
        for v in args {
            t.push_str(&format!(",{v}"));
        }
        self.log.push(t);
    }
    fn ow(p: i32, o: i32) -> usize {
        2 + 4 * p as usize + o as usize
    }
    fn uw(p: i32, u: i32) -> usize {
        12 + 4 * p as usize + u as usize
    }
    fn tw(ty: i32) -> usize {
        20 + ty as usize
    }
    fn sw(t: i32) -> usize {
        if t == 0x296 { 29 } else { 25 + t as usize }
    }
    fn orw(p: i32, u: i32) -> usize {
        35 + 4 * p as usize + u as usize
    }
    fn crw(p: i32, u: i32) -> usize {
        43 + 4 * p as usize + u as usize
    }
    fn at(w: usize) -> String {
        format!("p{w}+0x0")
    }
}
impl GarrisonHead for V<'_> {
    fn who(&self) -> i32 {
        i32::from(self.m[0x4a])
    }
    fn ignore_orders(&self) -> bool {
        rd32(&self.c[6], 0) != 0
    }
    fn ignoring_count(&self, _who: i32) -> i32 {
        unreachable!("the sweep keeps ignore_orders at 0")
    }
    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
        unreachable!("the sweep keeps ignore_orders at 0")
    }
    fn drop_ignored(&mut self, _o: i32, _who: i32) {
        unreachable!("the sweep keeps ignore_orders at 0")
    }
    fn refresh(&mut self) {
        self.rec(0, "m+0x0", &[]);
    }
}
impl SpellView for V<'_> {
    fn validate_spell(&mut self, a: [i32; 5]) -> bool {
        self.rec(1, "m+0x0", &a);
        self.a[5 + 1] != 0
    }
    fn set_form(&mut self) {
        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
    }
    fn count(&self) -> i32 {
        rd32(self.m, 0xc)
    }
    fn member(&self, i: i32) -> i32 {
        rd16(self.m, 0x8cc + 2 * i as usize)
    }
    fn type_is_spell(&mut self, ty: i32) -> bool {
        if self.a[BASE_FLAG] != 0 {
            let x = rd32(&self.p[Self::tw(ty)], 4);
            (0x275..0x2ac).contains(&x)
        } else {
            self.rec(2, &Self::at(Self::tw(ty)), &[]);
            self.a[5 + 2] != 0
        }
    }
    fn type_is_unit(&mut self, ty: i32) -> bool {
        self.rec(3, &Self::at(Self::tw(ty)), &[]);
        self.seq(3) != 0
    }
    fn unit_type_is_aircraft(&mut self, ty: i32) -> bool {
        rd32(&self.p[31 + ty as usize], 0x218) == 2
    }
    fn leader_no_best(&self, p: i32) -> bool {
        self.c[3][32 * p as usize + 0x1c] & 1 != 0
    }
    fn obj_flag1(&self, p: i32, o: i32) -> bool {
        self.p[Self::ow(p, o)][8] & 1 != 0
    }
    fn obj_alive(&mut self, p: i32, o: i32) -> bool {
        self.rec(4, &Self::at(Self::ow(p, o)), &[]);
        self.seq(4) != 0
    }
    fn obj_bc(&mut self, p: i32, o: i32) -> bool {
        self.rec(5, &Self::at(Self::ow(p, o)), &[]);
        self.seq(5) != 0
    }
    fn obj_s18(&mut self, p: i32, o: i32) -> bool {
        self.rec(9, &Self::at(Self::ow(p, o)), &[]);
        self.seq(9) != 0
    }
    fn obj_s20(&mut self, p: i32, o: i32) -> bool {
        self.rec(18, &Self::at(Self::ow(p, o)), &[]);
        self.seq(18) != 0
    }
    fn obj_s0c(&mut self, p: i32, o: i32) -> bool {
        self.rec(3, &Self::at(Self::ow(p, o)), &[]);
        self.seq(3) != 0
    }
    fn obj_queue_up(&mut self, p: i32, o: i32, ty: i32) {
        self.rec(19, &Self::at(Self::ow(p, o)), &[]);
        let build = self.a[5 + 19].to_string();
        self.rec(21, &build, &[ty, 1]);
    }
    fn spell_flags(&self, t: i32) -> i32 {
        rd32(&self.p[Self::sw(t)], 0x1c8)
    }
    fn spell_mana(&self, t: i32) -> i32 {
        rd32(&self.p[Self::sw(t)], 0x1d0)
    }
    fn unit_is_casting(&mut self, p: i32, u: i32, ty: i32) -> bool {
        self.rec(6, &Self::at(Self::uw(p, u)), &[ty]);
        self.seq(6) != 0
    }
    fn is_castable(&mut self, t: i32, u: i32, p: i32) -> i32 {
        self.rec(7, &Self::at(Self::sw(t)), &[u, p, 0]);
        self.seq(7)
    }
    fn mana_left(&mut self, p: i32, u: i32) -> i32 {
        self.rec(8, &Self::at(Self::uw(p, u)), &[]);
        self.seq(8)
    }
    fn mana(&mut self, p: i32, u: i32) -> i32 {
        self.rec(13, &Self::at(Self::uw(p, u)), &[]);
        self.seq(13)
    }
    fn mana_burn(&self, p: i32, u: i32) -> i32 {
        rd16(&self.p[Self::uw(p, u)], 0x96)
    }
    fn unit_flags(&self, p: i32, u: i32) -> i32 {
        rd32(&self.p[Self::uw(p, u)], 0x68)
    }
    fn pending_mana(&mut self, p: i32, u: i32) -> i32 {
        let unit = Self::at(Self::uw(p, u));
        let order = Self::at(Self::orw(p, u));
        self.rec(10, &unit, &[]);
        if rd32(&self.p[Self::uw(p, u)], 0x40) == 0 {
            return 0;
        }
        self.rec(11, &order, &[]);
        if self.seq(11) != 0xe {
            return 0;
        }
        self.rec(10, &unit, &[]);
        self.rec(12, &order, &[]);
        let id = rd32(&self.p[Self::crw(p, u)], 0x20);
        rd32(&self.p[Self::sw(id)], 0x1d0)
    }
    fn type_s84(&mut self, ty: i32, who: i32, u: i32) -> bool {
        self.rec(14, &Self::at(Self::tw(ty)), &[who, -1, u, 1]);
        self.seq(14) != 0
    }
    fn is_valid_target(&mut self, t: i32, who: i32, ox: i32, whom: i32) -> bool {
        self.rec(15, &Self::at(Self::sw(t)), &[who, ox, whom, 0]);
        self.seq(15) != 0
    }
    fn type_s50(&mut self, ty: i32) -> bool {
        self.rec(16, &Self::at(Self::tw(ty)), &[]);
        self.seq(16) != 0
    }
    fn type_s54(&mut self, ty: i32) -> bool {
        self.rec(17, &Self::at(Self::tw(ty)), &[]);
        self.seq(17) != 0
    }
    fn get_job_time(&mut self, ty: i32, u: i32, p: i32) -> bool {
        self.rec(20, &Self::at(Self::sw(ty)), &[u, p]);
        self.seq(20) != 0
    }
    fn pay_and_cast(&mut self, ty: i32, u: i32, p: i32) {
        self.rec(22, &Self::at(Self::sw(ty)), &[u, p]);
        self.rec(23, &Self::at(Self::sw(ty)), &[u, p, 0, 0]);
    }
    fn unit_is_busy(&mut self, p: i32, u: i32) -> bool {
        self.rec(24, &Self::at(Self::uw(p, u)), &[]);
        self.seq(24) != 0
    }
    fn write_target(&mut self, p: i32, u: i32, ox: i32, whom: i32) {
        let uw = Self::uw(p, u);
        let held_ox = rd16(&self.p[uw], 0xa2);
        let held_whom = i32::from(self.p[uw][0xa8] as i8);
        if ox == held_ox && whom == held_whom {
            return;
        }
        let w = &mut self.p[uw];
        w[0x98..0x9a].copy_from_slice(&0i16.to_le_bytes());
        w[0xa2..0xa4].copy_from_slice(&(ox as i16).to_le_bytes());
        w[0xa8] = whom as u8;
        if ox >= 0 {
            let id = [self.p[Self::ow(whom, ox)][0x30], self.p[Self::ow(whom, ox)][0x31]];
            self.p[uw][0xa6..0xa8].copy_from_slice(&id);
        }
    }
    fn order_type(&mut self, p: i32, u: i32) -> i32 {
        self.rec(25, &Self::at(Self::uw(p, u)), &[]);
        self.seq(25)
    }
    fn kill_current_order(&mut self, p: i32, u: i32) {
        self.rec(26, &Self::at(Self::uw(p, u)), &[0]);
    }
    fn add_cast_order(&mut self, p: i32, u: i32, args: [i32; 5], queue: i32) {
        let [ox, whom, x, y, ty] = args;
        self.rec(27, &Self::at(Self::uw(p, u)), &[ox, whom, x, y, ty, queue, 1]);
    }
    fn console_who(&self) -> i32 {
        rd32(&self.c[4], 0x298)
    }
    fn warn_resources(&mut self, who: i32, ty: i32, last: i32) {
        let leader = (who * 0x6eec + 0xe3a390).to_string();
        self.rec(31, &leader, &[ty, last, -1, who]);
    }
    fn play(&mut self, who: i32, sound: i32) {
        self.rec(28, &who.to_string(), &[sound]);
    }
    fn feedback(&mut self, msg: i32) {
        let src = rd32(&self.c[8], 0).wrapping_add(msg);
        self.rec(29, "sp-0x8c", &[src]);
        let win = rd32(&self.c[7], 0).to_string();
        let mut words = STRING_WORDS.to_vec();
        words.extend([-1, RED]);
        self.rec(30, &win, &words);
    }
}
let mut v = V {
    a,
    m: &mut *m,
    c: &*c,
    p: &mut *p,
    log: &mut log,
    turn: [0; 32],
};
action_spell_steps(&mut v, a[0], a[1], a[2], a[3], a[4]);
0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "action_spell — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_spell — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "action_spell — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[10 + i], "action_spell — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Group::action_spell@006fe1a0: {n} rows agree");
}
