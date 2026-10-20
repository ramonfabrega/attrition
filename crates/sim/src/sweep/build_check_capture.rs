//! `Build::check_capture@006276a0` under the emulator against the port (item 3356; tools/sweep_gen.py, obj with windows).

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

/// `Build::check_capture@006276a0`, run by `tools/emu/sweep_build_check_capture.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p23 sparse table 0 (8 rows × 3 slice(s), row-major), p24..p26 sparse table 1 (3 rows × 1 slice(s), row-major), p27..p87 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061f0 points at 312 random bytes; the global cell 0x00c061ec points at 1364 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 64 random bytes; 4 random bytes at 0x00c8cd00; callee 0x0062d1d0 (0 args) stubbed; callee 0x00461420 (2 args) stubbed; callee 0x004613d0 (4 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x00733380 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cd60 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x00642fc0 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; vtable slot +0x10c (1 args) stubbed; vtable slot +0x94 (1 args) stubbed; vtable slot +0x150 (3 args) stubbed; vtable slot +0x158 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p27+0` points at 96 random bytes less `0` (placed); the word at `p27+4` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p30+0` points at 96 random bytes less `0` (placed); the word at `p30+4` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p33+0` points at 96 random bytes less `0` (placed); the word at `p33+4` points at 96 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 128 random bytes less `0` (placed); the word at `p36+4` points at 128 random bytes less `0` (placed); the word at `p36+8` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 128 random bytes less `0` (placed); the word at `p40+4` points at 128 random bytes less `0` (placed); the word at `p40+8` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p44+0` points at 128 random bytes less `0` (placed); the word at `p44+4` points at 128 random bytes less `0` (placed); the word at `p44+8` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p48+0` points at 128 random bytes less `0` (placed); the word at `p48+4` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p51+0` points at 128 random bytes less `0` (placed); the word at `p51+4` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p54+0` points at 128 random bytes less `0` (placed); the word at `p54+4` points at 128 random bytes less `0` (placed); the word at `c2+0x134` points at 2816 random bytes less `0` (placed); the word at `p28+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p28+0x20` points at 112 random bytes less `0` (placed); the word at `p28+0x28` points at 32 random bytes less `0` (placed); the word at `p29+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p29+0x20` points at 112 random bytes less `0` (placed); the word at `p29+0x28` points at 32 random bytes less `0` (placed); the word at `p31+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p31+0x20` points at 112 random bytes less `0` (placed); the word at `p31+0x28` points at 32 random bytes less `0` (placed); the word at `p32+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p32+0x20` points at 112 random bytes less `0` (placed); the word at `p32+0x28` points at 32 random bytes less `0` (placed); the word at `p34+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p34+0x20` points at 112 random bytes less `0` (placed); the word at `p34+0x28` points at 32 random bytes less `0` (placed); the word at `p35+0x18` points at 56 random bytes less `0x1e4` (placed); the word at `p35+0x20` points at 112 random bytes less `0` (placed); the word at `p35+0x28` points at 32 random bytes less `0` (placed); the word at `p59+0x18` points at 16 random bytes less `0` (placed); the word at `p62+0x18` points at 16 random bytes less `0` (placed); the word at `p65+0x18` points at 16 random bytes less `0` (placed); the word at `p68+0x18` points at 16 random bytes less `0` (placed); the word at `p71+0x18` points at 16 random bytes less `0` (placed); the word at `p74+0x18` points at 16 random bytes less `0` (placed); the word at `p60+0x18` points at 8 random bytes less `0x74` (placed); the word at `p63+0x18` points at 8 random bytes less `0x74` (placed); the word at `p66+0x18` points at 8 random bytes less `0x74` (placed); the word at `p69+0x18` points at 8 random bytes less `0x74` (placed); the word at `p72+0x18` points at 8 random bytes less `0x74` (placed); the word at `p75+0x18` points at 8 random bytes less `0x74` (placed); slot +0x10c laid as 0x00645310 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 1 answers 21 values in call order, printed after the answers; stub 2 answers 21 values in call order, printed after the answers; stub 3 answers 10 values in call order, printed after the answers; stub 5 answers 12 values in call order, printed after the answers; stub 6 answers 12 values in call order, printed after the answers; stub 1 answers 0..3 every row; stub 2 answers 0..3 every row; stub 3 answers 0..4 every row; stub 4 answers 0..5 every row; stub 5 answers 0..2 every row; stub 6 answers 0..2 every row; stub 7 answers 0..2 every row; stub 23 answers 0..1 every row; stub 27 answers -1..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 writes a value in [0, 0, 0, 1, 1, 1, -1] through its argument 2, printed after the answers (`o` in a place); stub 2 writes a value in [0, 0, 0, 0, 0, 1, 2] through its argument 3, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); stub 17 answers from its own `ecx`: the word at `+0x30`; stub 18 answers from its own `ecx`: the word at `+0x34`; stub 19 answers from its own `ecx`: the word at `+0x38`; stub 20 answers from its own `ecx`: the word at `+0x3c`; stub 21 answers from its own `ecx`: the word at `+0x20`; stub 22 answers from its own `ecx`: the word at `+0x28`; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); 3 rows 0x6eec apart at `0xc061e0`, slices +0x74:0xc (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p76+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p80+0` as well as at `this`; the stubbed vtable laid at `p72+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_check_capture.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0]`, `a[1]` the arguments (the captor's index in `objects[who]`, and its player);
// `a[2 + i]` stub `i`'s answer; `a[32]`, `a[33]` the index and player `get_down` writes; then the seq
// values in flag order — `a[34..55]` `valid`'s 21, `a[55..76]` `get_down`'s 21, `a[76..86]`
// `valid_filter`'s 10, `a[86..98]` `is_ally`'s 12, `a[98..110]` `is_enemy`'s 12; then `a[110]`, whether
// slot `+0x10c` held `ObjectData::say_name` (a native read, the caller inlines it).
// Stubs: 0 `check_capture_eligible`, 1 `WorldData::valid`, 2 `get_down`, 3 `Search::valid_filter`, 4
// `ObjectData::num_inside`, 5 `is_ally`, 6 `is_enemy`, 7 `Cities::capture_city`, 8 `Object::die`, 9
// `SoundGlobal::play`, 10 `String::String`, 11 `String::operator+=`, 12 `operator=`, 13 `String::parse`, 14
// `String::close`, 15 `MessageWin::add_message`, 16 `Wall::mask_me`, then the slots `+0x18`, `+0x120`, `+0xec`,
// `+0x20`, `+0xac`, `+0xb0`, `+0xfc`, `+0x164`, `+0x1a8`, `+0x10c`, `+0x94`, `+0x150`, `+0x158` (17 to 29).
// Windows: `m` the building, `c0` the objects cell, `c1` `div_3_table`, `c2` the world, `c3` `constants`,
// `c4` `game`, `c5` the console, `c6` the message window, `c7` the word at `0xc8cd00`; `p3r + s` leader row
// `r`'s `+0`, `+8`, `+0x74`; `p24 + r` the constants leaders' `+0x74`; `p27 + 3k` the objects of player
// `k`, `p28 + 3k + j` object `j`; `p36 + 4k` the cities of `k`, `p37 + 4k + i` city `i`; `p48 + 3k` the units
// of `k`, `p49 + 3k + j` unit `j`; `p57` the tile array; for object `n = 2k + j`: `p58 + 3n` its type, `p59 +
// 3n` its `+0xac` record, `p60 + 3n` its `+0xb0` record; `p76 + n` the `+0xac` record's type, `p82 + n` the
// `+0xb0` record's name.
//
// **The port is the crate's own steps**, `crate::capture::check_capture_steps`, which `Sim::check_capture`
// calls over its `SimCapture`; here the view is the emulated original's windows.
use crate::capture::{CaptureView, check_capture_steps};
const ROW: i32 = 0x00e3_a390;
const MSG: i32 = 0x00ed_6730;
fn r16(m: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([m[off], m[off + 1]]))
}
fn pw(win: usize, off: usize) -> String {
    format!("p{win}+{off:#x}")
}
struct V<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    nv: usize,
    ng: usize,
    nf: usize,
    na: usize,
    ne: usize,
}
impl V<'_> {
    fn obj(h: (i32, i32)) -> usize {
        28 + 3 * h.0 as usize + h.1 as usize
    }
    fn n(h: (i32, i32)) -> usize {
        2 * h.0 as usize + h.1 as usize
    }
    fn seq(&self, base: usize, len: usize, used: usize) -> i32 {
        self.a[base + used.min(len - 1)]
    }
    fn city_win(&self, who: i32, i: i32) -> usize {
        37 + 4 * who as usize + i as usize
    }
    fn my_city(&self) -> usize {
        self.city_win(i32::from(self.m[9]), r16(self.m, 0x72))
    }
}
impl CaptureView for V<'_> {
    type H = (i32, i32);
    fn eligible(&mut self) -> bool {
        self.log.push("s0:m+0x0".into());
        self.a[2] != 0
    }
    fn inside_down(&mut self) -> i32 {
        r16(self.m, 0x28)
    }
    fn eject_pending(&mut self) {
        let h = (i32::from(self.m[9]), r16(self.m, 0xa));
        self.log.push(format!("s21:{}", pw(Self::obj(h), 0)));
        let ac = 59 + 3 * Self::n(h);
        let v = u16::from_le_bytes([self.p[ac][0x60], self.p[ac][0x61]]) | 0x4000;
        self.p[ac][0x60..0x62].copy_from_slice(&v.to_le_bytes());
    }
    fn owner(&mut self) -> i32 {
        i32::from(self.m[9])
    }
    fn building(&mut self) -> (i32, i32) {
        (i32::from(self.m[9]), r16(self.m, 0xa))
    }
    fn centre(&mut self) -> bool {
        self.m[8] & 0x20 != 0
    }
    fn building_pos(&mut self) -> (i32, i32) {
        (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
    }
    fn div3(&mut self, i: i32) -> i32 {
        rd(&self.c[1], 4 * i as usize)
    }
    fn leader_id(&mut self, who: i32) -> i32 {
        rd(&self.p[3 * who as usize + 1], 0)
    }
    fn leader_flags(&mut self, who: i32) -> i32 {
        rd(&self.p[3 * who as usize], 0)
    }
    fn relation(&mut self, who: i32, other: i32) -> i32 {
        rd(&self.p[3 * who as usize + 2], 4 * other as usize)
    }
    fn relation_const(&mut self, who: i32, leader: i32) -> i32 {
        rd(&self.p[24 + who as usize], 4 * leader as usize)
    }
    fn is_ally(&mut self, who: i32, other: i32) -> bool {
        self.log.push(format!("s5:{},{other}", pw(3 * who as usize, 0)));
        let r = self.seq(86, 12, self.na);
        self.na += 1;
        r != 0
    }
    fn is_enemy(&mut self, who: i32, other: i32) -> bool {
        self.log.push(format!("s6:{},{other}", pw(3 * who as usize, 0)));
        let r = self.seq(98, 12, self.ne);
        self.ne += 1;
        r != 0
    }
    fn handle(&mut self, who: i32, idx: i32) -> (i32, i32) {
        (who, idx)
    }
    fn kind(&mut self, h: (i32, i32)) -> i32 {
        rd(&self.p[58 + 3 * Self::n(h)], 0x34)
    }
    fn is_unit(&mut self, h: (i32, i32)) -> bool {
        self.log.push(format!("s17:{}", pw(Self::obj(h), 0)));
        rd(&self.p[Self::obj(h)], 0x30) != 0
    }
    fn decoy(&mut self, h: (i32, i32)) -> bool {
        rd(&self.p[49 + 3 * h.0 as usize + h.1 as usize], 0x68) & 1 != 0
    }
    fn can_capture(&mut self, h: (i32, i32)) -> bool {
        self.log.push(format!("s18:{}", pw(Self::obj(h), 0)));
        rd(&self.p[Self::obj(h)], 0x34) != 0
    }
    fn missile(&mut self, h: (i32, i32)) -> bool {
        rd(&self.p[58 + 3 * Self::n(h)], 0) & 0x800_0000 != 0
    }
    fn in_use(&mut self, h: (i32, i32)) -> bool {
        rd(&self.p[Self::obj(h)], 8) & 1 != 0
    }
    fn player(&mut self, h: (i32, i32)) -> i32 {
        h.0
    }
    fn valid_filter(&mut self, h: (i32, i32)) -> bool {
        self.log.push(format!("s3:{},{},-1,-1,8", h.0, h.1));
        let r = self.seq(76, 10, self.nf);
        self.nf += 1;
        r != 0
    }
    fn pos(&mut self, h: (i32, i32)) -> (i32, i32) {
        let o = &self.p[Self::obj(h)];
        (rd(o, 0x10) ^ 0x63637, rd(o, 0x14) ^ 0x63637)
    }
    fn value(&mut self, h: (i32, i32)) -> i32 {
        self.log.push(format!("s19:{}", pw(Self::obj(h), 0)));
        rd(&self.p[Self::obj(h)], 0x38)
    }
    fn is_building(&mut self, h: (i32, i32)) -> bool {
        self.log.push(format!("s20:{}", pw(Self::obj(h), 0)));
        rd(&self.p[Self::obj(h)], 0x3c) != 0
    }
    fn garrison_weight(&mut self, h: (i32, i32)) -> i32 {
        let (o, n) = (Self::obj(h), Self::n(h));
        self.log.push(format!("s21:{}", pw(o, 0)));
        self.log.push(format!("s23:{}", pw(76 + n, 0)));
        let fort = self.a[2 + 23] != 0;
        self.log.push(format!("s21:{}", pw(o, 0)));
        self.log.push(format!("s4:{},1", pw(59 + 3 * n, 0)));
        self.a[2 + 4] + if fort { 12 } else { 6 }
    }
    fn reveal(&mut self, h: (i32, i32), who: i32) {
        let (o, n) = (Self::obj(h), Self::n(h));
        let ac = 59 + 3 * n;
        self.log.push(format!("s21:{}", pw(o, 0)));
        let seen = i32::from(self.p[ac][0x40] as i8);
        if (1i32.wrapping_shl(who as u32 & 31)) & seen == 0 {
            self.log.push(format!("s21:{}", pw(o, 0)));
            self.p[ac][0x40] = (seen | 1i32.wrapping_shl(who as u32 & 31)) as u8;
            self.log.push(format!("s21:{}", pw(o, 0)));
            self.log.push(format!("s24:{}", pw(ac, 0)));
        }
    }
    fn city_stamp(&mut self) -> i32 {
        rd(&self.p[self.my_city()], 0x20)
    }
    fn city_strength(&mut self) -> i32 {
        rd(&self.p[self.my_city()], 0x28)
    }
    fn city_index(&mut self) -> i32 {
        r16(self.m, 0x72)
    }
    fn city_founder(&mut self) -> i32 {
        i32::from(self.p[self.my_city()][0x60] as i8)
    }
    fn city_race(&mut self) -> i32 {
        i32::from(self.p[self.my_city()][0x5f] as i8)
    }
    fn frame(&mut self) -> i32 {
        rd(&self.c[4], 0x550)
    }
    fn city_capture_radius(&mut self) -> i32 {
        rd(&self.c[3], 0x134)
    }
    fn unit_respond_range(&mut self) -> i32 {
        rd(&self.c[3], 0x18)
    }
    fn valid_tile(&mut self, _x: i32, _y: i32) -> bool {
        self.log.push("s1:c2+0x0,sp-0x28,sp-0x2c".into());
        let r = self.seq(34, 21, self.nv);
        self.nv += 1;
        r != 0
    }
    fn get_down(&mut self, _x: i32, _y: i32) -> (bool, Option<(i32, i32)>) {
        self.log
            .push("s2:c2+0x0,sp-0x28,sp-0x2c,sp-0x38,sp-0x48".into());
        let r = self.seq(55, 21, self.ng);
        self.ng += 1;
        (r != 0, (self.a[32] >= 0).then_some((self.a[33], self.a[32])))
    }
    fn tile_region(&mut self, x: i32, y: i32) -> u16 {
        let at = (rd(&self.c[2], 0).wrapping_mul(y).wrapping_add(x)) as usize * 0x1c + 4;
        u16::from_le_bytes([self.p[57][at], self.p[57][at + 1]])
    }
    fn next(&mut self, h: (i32, i32)) -> Option<(i32, i32)> {
        let o = &self.p[Self::obj(h)];
        (r16(o, 0x2c) >= 0).then(|| (r16(o, 0x2e), r16(o, 0x2c)))
    }
    fn capture_city(&mut self, taker: i32, city: i32, owner: i32, ecx: i32) -> i32 {
        // The stale `ecx`: a leaders row (laid in the sparse table's window) or the city's number.
        let ecx = if ecx >= ROW && (ecx - ROW) % 0x6eec == 0 {
            pw(3 * ((ecx - ROW) / 0x6eec) as usize, 0)
        } else {
            ecx.to_string()
        };
        self.log.push(format!("s7:{ecx},{taker},{city},{owner}"));
        self.a[2 + 7]
    }
    fn set_city_strength(&mut self, taker: i32, city: i32, v: i32) {
        let w = self.city_win(taker, city);
        wr(&mut self.p[w], 0x28, v);
    }
    fn swap_team(&mut self, who: i32) -> i32 {
        self.log.push(format!("s27:m+0x0,{who}"));
        self.a[2 + 27]
    }
    fn die(&mut self) {
        if rd(self.m, 0) == 0xb4_2174 {
            self.log.push("s8:m+0x0,0,-1,0".into());
        } else {
            self.log.push("s29:m+0x0,0,-1,0".into());
        }
    }
    fn console_who(&mut self) -> i32 {
        rd(&self.c[5], 0x298)
    }
    fn play(&mut self, sound: i32) {
        self.log.push(format!("s9:c5+0x0,{sound}"));
    }
    fn announce(&mut self, new: i32, who: i32) {
        let h = (who, new);
        let (o, n) = (Self::obj(h), Self::n(h));
        self.log.push("s10:sp-0x78".into());
        self.log.push(format!("s22:{}", pw(o, 0)));
        let data = 60 + 3 * n;
        if self.a[110] != 0 {
            self.log.push(format!("s11:sp-0x78,{}", pw(82 + n, 0)));
        } else {
            self.log.push(format!("s26:{},sp-0x78", pw(data, 0)));
        }
        self.log
            .push(format!("s12:{MSG},{}", rd(&self.c[7], 0).wrapping_add(0x76c)));
        self.log.push(format!("s13:{MSG},sp-0x8c,sp-0x78"));
        self.log.push(format!("s12:{MSG},{}", self.a[2 + 13]));
        self.log.push("s14:sp-0x8c".into());
        let ob = &self.p[o];
        let (x, y) = (rd(ob, 0x10) ^ 0x63637, rd(ob, 0x14) ^ 0x63637);
        let list = pw(27 + 3 * who as usize, 0);
        self.log.push(format!(
            "s15:c6+0x0,{MSG},{x},{y},0,{},-1,{list},0,0,{list}",
            0x00c8_d248
        ));
        self.log.push("s14:sp-0x78".into());
    }
    fn activate_new(&mut self, new: i32, who: i32) {
        let h = (who, new);
        self.log.push(format!("s22:{}", pw(Self::obj(h), 0)));
        self.log
            .push(format!("s25:{},0,1,0", pw(60 + 3 * Self::n(h), 0)));
    }
    fn close_old(&mut self) {
        self.log.push("s28:m+0x0,0,-1,0".into());
    }
    fn mask_new(&mut self, new: i32, who: i32) {
        let h = (who, new);
        self.log.push(format!("s22:{}", pw(Self::obj(h), 0)));
        self.log
            .push(format!("s16:{},1,2", pw(60 + 3 * Self::n(h), 0)));
    }
}
let mut v = V {
    a,
    m,
    c,
    p,
    log: &mut log,
    nv: 0,
    ng: 0,
    nf: 0,
    na: 0,
    ne: 0,
};
i64::from(check_capture_steps(&mut v, a[0], a[1]))
        };
        assert_eq!(got, want_eax, "check_capture — eax, row `{line}`");
        assert_eq!(log, want_log, "check_capture — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_capture — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_capture — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "check_capture — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Build::check_capture@006276a0: {n} rows agree");
}
