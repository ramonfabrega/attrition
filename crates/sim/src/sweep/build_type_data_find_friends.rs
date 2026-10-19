//! `BuildTypeData::find_friends@00639270` under the emulator against the port (item 3339; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

use crate::build::Ident;

use crate::friends::{FriendsView, find_friends_steps};

/// `Build::vftable` (`0xb42174`): the vtable word a record is compared with.
const BUILD_VFTABLE: i32 = 0x00b4_2174;

/// The windows' first answer-record: `p6`, one a `vtable[0x3c]` call, the last repeated.
const FIRST_RECORD: usize = 6;

const RECORDS: usize = 48;

/// The type index the original's `is(code, 0)` is asked with, per ident.
const fn code(id: Ident) -> i32 {
    match id {
        Ident::Tower => 0x1b7,
        Ident::Lookout => 0x209,
        Ident::Woodcutter => 0x1a2,
        Ident::Farm => 0x1a1,
        Ident::University => 0x1a4,
        Ident::Granary => 0x1a7,
        _ => -1,
    }
}

/// `BuildTypeData::get_enhancing_good@00639880`: a switch on `[this+4]` (no crate
/// counterpart over a raw type index; `ai_place::enhancing_good` is by ident).
fn enhancing_good_of(ty: i32) -> i32 {
    match ty {
        0x1a7 => 0,
        0x1a8 => 1,
        0x1a9 => 4,
        0x1aa => 5,
        _ => -1,
    }
}

/// `BuildTypeData::get_good@0063bd50`: a switch on `[this+4]`.
fn good_of(ty: i32) -> i32 {
    match ty {
        0x1a1 => 0,
        0x1a2 => 1,
        0x1a3 => 4,
        0x1a4 => 3,
        0x1a5 | 0x1a6 => 5,
        _ => -1,
    }
}

/// The emulated original's row, answered to [`find_friends_steps`] call by call.
/// `a[4..12]` the stubs' single answers (unread), then each stub's `--seq` run:
/// `is` (stub 0) `a[12..84]`, the record's `is` (2) `a[84..108]`, gathers (3)
/// `a[108..132]`, wonder (4) `a[132..156]`, `is_gather_enhancer` (5) `a[156..180]`,
/// `is_military_trainer` (6) `a[180..204]`, `find_building_placed_at` (7)
/// `a[204..212]`; the last of a run repeats.
struct Emu<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a [Vec<u8>],
    p: &'a [Vec<u8>],
    n: [usize; 8],
    l: [Vec<String>; 8],
}

impl Emu<'_> {
    fn ans(&mut self, stub: usize, base: usize, len: usize) -> i32 {
        let i = self.n[stub].min(len - 1);
        self.n[stub] += 1;
        self.a[base + i]
    }
    /// One `objects[who][nb]->vtable[0x3c]()`: the record window it answers.
    fn record(&mut self, nb: i32) -> usize {
        let k = self.n[1].min(RECORDS - 1);
        self.n[1] += 1;
        self.l[1].push(format!("p{}+0x0", 1 + nb));
        FIRST_RECORD + k
    }
}

impl FriendsView for Emu<'_> {
    fn attack(&self) -> bool {
        rd(self.m, 0x1e8) != 0
    }
    fn this_is(&mut self, id: Ident) -> bool {
        self.l[0].push(format!("m+0x0,{},0", code(id)));
        self.ans(0, 12, 72) != 0
    }
    fn no_city(&self) -> bool {
        rd(self.m, 0x2c0) & 0x10 != 0
    }
    fn world_cells(&self) -> (i32, i32) {
        (rd(&self.c[0], 0), rd(&self.c[0], 4))
    }
    fn placed_at(&mut self, tx: i32, ty: i32) -> i32 {
        let who = self.a[3];
        self.l[7].push(format!("12626800,{tx},{ty},{who},-1,-1"));
        self.ans(7, 204, 8)
    }
    fn nb_city(&mut self, nb: i32) -> i32 {
        let k = self.record(nb);
        i32::from(i16::from_le_bytes([self.p[k][0x72], self.p[k][0x73]]))
    }
    fn this_gather_enhancer(&mut self) -> bool {
        self.l[5].push("m+0x0".to_string());
        self.ans(5, 156, 24) != 0
    }
    fn this_enhancing_good(&mut self) -> i32 {
        enhancing_good_of(rd(self.m, 4))
    }
    fn nb_good(&mut self, nb: i32) -> i32 {
        self.record(nb);
        good_of(rd(&self.p[5], 4))
    }
    fn this_military_trainer(&mut self) -> bool {
        self.l[6].push("m+0x0".to_string());
        self.ans(6, 180, 24) != 0
    }
    fn nb_military_trainer(&mut self, nb: i32) -> bool {
        self.record(nb);
        self.l[6].push("p5+0x0".to_string());
        self.ans(6, 180, 24) != 0
    }
    fn nb_flag20(&mut self, nb: i32) -> bool {
        let k = self.record(nb);
        self.p[k][8] & 0x20 != 0
    }
    fn nb_is(&mut self, nb: i32, id: Ident) -> bool {
        let k = self.record(nb);
        if rd(&self.p[k], 0) == BUILD_VFTABLE {
            self.l[0].push(format!("p5+0x0,{},0", code(id)));
            self.ans(0, 12, 72) != 0
        } else {
            self.l[2].push(format!("p{k}+0x0,{},0", code(id)));
            self.ans(2, 84, 24) != 0
        }
    }
    fn nb_gathers(&mut self, nb: i32) -> bool {
        self.record(nb);
        self.l[3].push("p5+0x0".to_string());
        self.ans(3, 108, 24) != 0
    }
    fn nb_gather_enhancer(&mut self, nb: i32) -> bool {
        self.record(nb);
        self.l[5].push("p5+0x0".to_string());
        self.ans(5, 156, 24) != 0
    }
    fn nb_wonder(&mut self, nb: i32) -> bool {
        self.record(nb);
        self.l[4].push("p5+0x0".to_string());
        self.ans(4, 132, 24) != 0
    }
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `BuildTypeData::find_friends@00639270`, run by `tools/emu/sweep_build_type_data_find_friends.py` under unicorn
/// (tools/emu): 720 random bytes of object, 4 argument(s), `p` windows in order: p0..p5 the place(s), in flag order, p6..p53 the stubs' answer window(s); the global cell 0x00c061d0 points at 16 random bytes; vtable slot +0x60 (2 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; callee 0x00472b50 (0 args) stubbed; callee 0x0063bcf0 (0 args) stubbed; callee 0x00658c80 (5 args) stubbed; the word at `0xc0ab84+2*0x1c` points at 20 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 0, 1] (90%); stub 6 answered from [0, 0, 1] (90%); stub 1 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p6+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p7+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p8+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p9+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p10+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p11+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p12+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p13+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p14+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p15+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p16+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p17+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p18+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p19+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p20+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p21+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p22+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p23+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p24+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p25+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p26+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p27+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p28+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p29+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p30+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p31+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p32+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p33+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p34+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p35+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p36+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p38+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p39+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p40+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p41+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p42+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p43+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p44+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p45+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p46+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p47+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p48+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p49+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p50+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p51+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p52+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p53+0` laid as the image vtable at 11805044 (the stub slots copied there) half the rows; the word at `p6+0x18` laid as the address of `p5` every row; the word at `p7+0x18` laid as the address of `p5` every row; the word at `p8+0x18` laid as the address of `p5` every row; the word at `p9+0x18` laid as the address of `p5` every row; the word at `p10+0x18` laid as the address of `p5` every row; the word at `p11+0x18` laid as the address of `p5` every row; the word at `p12+0x18` laid as the address of `p5` every row; the word at `p13+0x18` laid as the address of `p5` every row; the word at `p14+0x18` laid as the address of `p5` every row; the word at `p15+0x18` laid as the address of `p5` every row; the word at `p16+0x18` laid as the address of `p5` every row; the word at `p17+0x18` laid as the address of `p5` every row; the word at `p18+0x18` laid as the address of `p5` every row; the word at `p19+0x18` laid as the address of `p5` every row; the word at `p20+0x18` laid as the address of `p5` every row; the word at `p21+0x18` laid as the address of `p5` every row; the word at `p22+0x18` laid as the address of `p5` every row; the word at `p23+0x18` laid as the address of `p5` every row; the word at `p24+0x18` laid as the address of `p5` every row; the word at `p25+0x18` laid as the address of `p5` every row; the word at `p26+0x18` laid as the address of `p5` every row; the word at `p27+0x18` laid as the address of `p5` every row; the word at `p28+0x18` laid as the address of `p5` every row; the word at `p29+0x18` laid as the address of `p5` every row; the word at `p30+0x18` laid as the address of `p5` every row; the word at `p31+0x18` laid as the address of `p5` every row; the word at `p32+0x18` laid as the address of `p5` every row; the word at `p33+0x18` laid as the address of `p5` every row; the word at `p34+0x18` laid as the address of `p5` every row; the word at `p35+0x18` laid as the address of `p5` every row; the word at `p36+0x18` laid as the address of `p5` every row; the word at `p37+0x18` laid as the address of `p5` every row; the word at `p38+0x18` laid as the address of `p5` every row; the word at `p39+0x18` laid as the address of `p5` every row; the word at `p40+0x18` laid as the address of `p5` every row; the word at `p41+0x18` laid as the address of `p5` every row; the word at `p42+0x18` laid as the address of `p5` every row; the word at `p43+0x18` laid as the address of `p5` every row; the word at `p44+0x18` laid as the address of `p5` every row; the word at `p45+0x18` laid as the address of `p5` every row; the word at `p46+0x18` laid as the address of `p5` every row; the word at `p47+0x18` laid as the address of `p5` every row; the word at `p48+0x18` laid as the address of `p5` every row; the word at `p49+0x18` laid as the address of `p5` every row; the word at `p50+0x18` laid as the address of `p5` every row; the word at `p51+0x18` laid as the address of `p5` every row; the word at `p52+0x18` laid as the address of `p5` every row; the word at `p53+0x18` laid as the address of `p5` every row; stub 0 answers 72 values in call order, printed after the answers; stub 2 answers 24 values in call order, printed after the answers; stub 3 answers 24 values in call order, printed after the answers; stub 4 answers 24 values in call order, printed after the answers; stub 5 answers 24 values in call order, printed after the answers; stub 6 answers 24 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 7 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_find_friends.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `BuildTypeData::find_friends@00639270(x, y, city, who)`: the crate's
            // `find_friends_steps` over [`Emu`], which answers each question from the
            // row's windows and the stubs' answers in the order they were asked.
            let mut v = Emu {
                a,
                m: &*m,
                c: &*c,
                p: &*p,
                n: [0; 8],
                l: Default::default(),
            };
            let r = find_friends_steps(&mut v, a[0], a[1], a[2]);
            for t in v.l {
                log.push(if t.is_empty() { "-".to_string() } else { t.join("/") });
            }
            i64::from(r)
        };
        assert_eq!(got, want_eax, "find_friends — eax, row `{line}`");
        assert_eq!(log, want_log, "find_friends — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_friends — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_friends — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "find_friends — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("BuildTypeData::find_friends@00639270: {n} rows agree");
}
