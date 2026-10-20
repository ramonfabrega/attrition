//! `Unit::plunder@00605b00` under the emulator against the port (item 3356; tools/sweep_gen.py, obj with windows).

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

/// `Unit::plunder@00605b00`, run by `tools/emu/sweep_unit_plunder.py` under unicorn
/// (tools/emu): 128 random bytes of object, 4 argument(s), `p` windows in order: p0..p11 sparse table 0 (2 rows × 6 slice(s), row-major), p12..p27 the place(s), in flag order, p28 stub 8's answer window; the global cell 0x00c061f0 points at 3200 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 64 random bytes; the global cell 0x00c061ac points at 64 random bytes; the global cell 0x00c061fc points at 64 random bytes; the global cell 0x00e85ddc points at 256 random bytes; the global cell 0x00c06184 points at 16 random bytes; 4 random bytes at 0x00c8cd00; 1 random bytes at 0x00cb195a (each byte in 0..1); 64 random bytes at 0x00cc2328; callee 0x0073a1b0 (6 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1d180 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x00a16120 (2 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00595260 (0 args) stubbed; the import cell 0x00ac5480 holds a stub (4 args); vtable slot +0xb8 (2 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; vtable slot +0x78 (7 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 96 random bytes less `0` (placed); the word at `p12+4` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 96 random bytes less `0` (placed); the word at `p15+4` points at 96 random bytes less `0` (placed); the word at `p13+0x18` points at 768 random bytes less `0` (placed); the word at `p14+0x18` points at 768 random bytes less `0` (placed); the word at `p16+0x18` points at 768 random bytes less `0` (placed); the word at `p17+0x18` points at 768 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `p5+0` points at 224 random bytes less `0` (placed); the word at `p11+0` points at 224 random bytes less `0` (placed); the word at `c4+0x10` points at 64 random bytes less `0` (placed); the word at `c5+0x10` points at 256 random bytes less `0` (placed); the word at `c6+0xc8` points at 32 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; stub 0 answered from [-1, -1, 0, 1, 5] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [1, 1, 1, 0, 0] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [8, 9, 9, 9, 10] (90%); stub 16 answered from [0, 1] (90%); stub 17 answered from [0, 1] (90%); stub 18 answered from [0, 1] (90%); stub 19 answered from [0, 1] (90%); stub 20 answered from [0, 1] (90%); stub 21 answered from [0, 1] (90%); stub 8 answers the address of 64 random bytes, a `p` window after the places; the word at `p26+0` laid as the address of `p18` every row; the word at `p26+4` laid as the address of `p19` every row; the word at `p26+8` laid as the address of `p20` every row; the word at `p26+0xc` laid as the address of `p21` every row; the word at `p26+0xc8` laid as the address of `p20` every row; stub 0 answers 2 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 12 answers 6 values in call order, printed after the answers; stub 23 answers 6 values in call order, printed after the answers; stub 7 answers 1..9 every row; stub 12 answers 0..65535 every row; stub 22 answers 0..3 every row; stub 23 answers -50..3000 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 15 is `__cdecl`: the caller pops its arguments; the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x914:0x4, +0x59bc:0x4, +0x59d8:0x4, +0x6928:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_plunder.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(11);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0..4]` the arguments (`p1`..`p4`), `a[4 + k]` stub `k`'s answer, then the seq values
// in flag order: `a[28..30]` `find_hero`'s, `a[30..36]` `type_avail`'s, `a[36..42]`
// `Random::get`'s, `a[42..48]` the price slot's; then `a[48..52]` whether slot `+0xb8`, `+0xc4`,
// `+0xcc`, `+0xd0` held the native function (`ObjectData::is`, `is_hero`, `is_supply`,
// `is_caravan`) and so was inlined. Stubs: 0 `find_hero`, 1 `tribe_can_type`, 2 `type_avail`,
// 3 `String += String`, 4 `String += int`, 5 `String =`, 6 `String::close`, 7 `String::reinit`,
// 8 `Recycler::pop` (not recorded: its `ecx` is a register the stubbed callees left), 9
// `get_neon`, 10 `add_message`, 11 `SoundGlobal::play`, 12 `Random::get`, 13 `has_tribe_bonus`,
// 14 `get_patch_version`, 15 `_itow_s`, then the slots `+0xb8` (16), `+0xc4` (17), `+0xcc` (18),
// `+0xd0` (19), `+0x60` (20), `+0x130` (21), `+0x68` (22), `+0x78` (23). Windows: `m` the
// unit, `c0` `constants`, `c1` `game`, `c2` the console, `c3` the message window, `c4`
// `goodtypes`, `c5` `unittypes`, `c6` `types.list`, `c7` `game_random`, `c8` the localised
// string table's base, `c9` `ScenarioData::plunder`, `c10` `String::buff`; `p[6r..6r+6]` leader
// `r`'s `+8`, `+0x914`, `+0x59bc`, `+0x59d8`, `+0x6928` and `+0x6eb8`; `p12` `units[0]` and
// `p13`/`p14` its two units, `p15` `units[1]` and `p16`/`p17`; `p18..p21` their type records;
// `p22` the unit's own type record; `p23`/`p24` the buckets; `p25` the goods' array; `p26` the
// unit types' array (entries 0 to 3 name `p18` to `p21`, entry `0x32` names `p20`); `p27` the
// record `types[0x32]`; `p28` the text bubble.
//
// **The port is the crate's own steps**, `crate::plunder::plunder_steps`, which
// `Sim::unit_plunder` calls over its `SimPlunder`; here the view is the emulated original's
// windows. `LeaderDataEncrypt::scratch` is the bucket write's twin and nothing of the row
// reads it. `wcscpy_s` is not reached: no stub can lay the pointer in the caller's local
// that `String::reinit` would, so `reinit` answers non-zero (parked: a stub that writes
// through its `ecx`).
use crate::plunder::{PlunderView, Str, plunder_steps};
struct V<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    n0: usize,
    n2: usize,
    n12: usize,
    n23: usize,
}
impl V<'_> {
    fn leader(who: i32) -> i32 {
        0xe3a390 + who * 0x6eec
    }
    fn row(who: i32) -> usize {
        6 * who as usize
    }
    fn unit(who: i32, idx: i32) -> usize {
        13 + 3 * who as usize + idx as usize
    }
    fn tw(who: i32, idx: i32) -> usize {
        18 + 2 * who as usize + idx as usize
    }
    fn s(s: Str) -> &'static str {
        match s {
            Str::A => "sp-0x5c",
            Str::B => "sp-0x70",
        }
    }
    fn tc(&self, who: i32) -> i32 {
        0xe39ce0 + 100 * i32::from(self.p[Self::row(who) + 4][0])
    }
    fn loc(&self) -> i32 {
        rd(&self.c[8], 0)
    }
    fn f8(&self, who: i32) -> i32 {
        rd(&self.p[Self::row(who)], 0)
    }
}
impl PlunderView for V<'_> {
    type Ty = usize;
    type Bub = usize;
    fn flags68(&mut self) -> i32 {
        rd(self.m, 0x68)
    }
    fn this_who(&mut self) -> i32 {
        i32::from(self.m[9])
    }
    fn this_index(&mut self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
    }
    fn this_coord(&mut self) -> (i32, i32) {
        (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
    }
    fn this_redirects(&mut self) -> bool {
        rd(&self.p[22], 0x2b4) & 0x10 != 0
    }
    fn this_redirect(&mut self) -> (i32, i32) {
        (
            i32::from(self.m[0x3e] as i8),
            i32::from(i16::from_le_bytes([self.m[0x28], self.m[0x29]])),
        )
    }
    fn thedespot_unit_plunder(&mut self) -> i32 {
        rd(&self.c[0], 0xc74)
    }
    fn general_plunder(&mut self) -> i32 {
        rd(&self.c[0], 0x3b8)
    }
    fn caravan_plunder(&mut self) -> i32 {
        rd(&self.c[0], 0x3b4)
    }
    fn aztec_slay_bonus(&mut self) -> i32 {
        rd(&self.c[0], 0x56c)
    }
    fn aztec_slay_cap(&mut self) -> i32 {
        rd(&self.c[0], 0x570)
    }
    fn aztec_plunder(&mut self) -> i32 {
        rd(&self.c[0], 0x578)
    }
    fn inca_refund(&mut self) -> i32 {
        rd(&self.c[0], 0x5a0)
    }
    fn hero_count(&mut self, who: i32, ty: i32) -> i32 {
        let r = Self::row(who);
        let (w, o) = if ty == 0x160 { (r + 2, 2) } else { (r + 3, 0) };
        i32::from(i16::from_le_bytes([self.p[w][o], self.p[w][o + 1]]))
    }
    fn find_hero(&mut self, x: i32, y: i32, who: i32, ty: i32) -> i32 {
        self.log
            .push(format!("s0:{x},{x},{y},{},0,{ty},0", self.f8(who)));
        let r = self.a[28 + self.n0.min(1)];
        self.n0 += 1;
        r
    }
    fn type_avail(&mut self, who: i32, good: i32) -> bool {
        self.log.push(format!("s2:{},{good},1", Self::leader(who)));
        let r = self.a[30 + self.n2.min(5)];
        self.n2 += 1;
        r != 0
    }
    fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
        self.log.push(format!("s13:{},{bonus}", Self::leader(who)));
        self.a[4 + 13] != 0
    }
    fn bucket(&mut self, who: i32, good: i32) -> i32 {
        rd(&self.p[23 + who as usize], 4 * good as usize) ^ 0x8221
    }
    fn slay_count(&mut self, who: i32) -> i32 {
        rd(&self.p[23 + who as usize], 0xdc) ^ 0x62766
    }
    fn add_good(&mut self, who: i32, good: i32, n: i32) {
        let w = 23 + who as usize;
        let new = (rd(&self.p[w], 4 * good as usize) ^ 0x8221).wrapping_add(n);
        wr(&mut self.p[w], 4 * good as usize, new ^ 0x8221);
    }
    fn plunder_scale(&mut self, who: i32) -> u32 {
        rd(&self.p[Self::row(who) + 1], 0) as u32
    }
    fn unit_type(&mut self, who: i32, idx: i32) -> usize {
        Self::tw(who, idx)
    }
    fn unit_is_42(&mut self, who: i32, idx: i32, ty: usize) -> bool {
        if self.a[48] != 0 {
            self.log.push(format!("s20:p{ty}+0x0"));
            self.a[4 + 20] != 0
        } else {
            self.log
                .push(format!("s16:p{}+0x0,66,0", Self::unit(who, idx)));
            self.a[4 + 16] != 0
        }
    }
    fn unit_cached_type(&mut self, who: i32, idx: i32) -> i32 {
        rd(&self.p[Self::unit(who, idx)], 0x54)
    }
    fn set_unit_cached_type(&mut self, who: i32, idx: i32, t: i32) {
        wr(&mut self.p[Self::unit(who, idx)], 0x54, t);
    }
    fn tribe_default_type(&mut self, who: i32) -> i32 {
        self.log.push(format!("s1:{},p27+0x0", Self::leader(who)));
        if self.a[4 + 1] != 0 {
            0x32
        } else {
            self.log.push(format!("s22:p27+0x0,{}", self.f8(who)));
            self.a[4 + 22]
        }
    }
    fn unittype_record(&mut self, i: i32) -> usize {
        // The array's entries are laid as the addresses of the unit types' windows.
        match i {
            0..=3 => 18 + i as usize,
            0x32 => 20,
            _ => unreachable!("an entry nothing laid: {i}"),
        }
    }
    fn price(&mut self, ty: usize, good: i32, who: i32) -> i32 {
        self.log
            .push(format!("s23:p{ty}+0x0,{good},{who},-1,-1,1,1,-1"));
        let r = self.a[42 + self.n23.min(5)];
        self.n23 += 1;
        r
    }
    fn unit_is_hero(&mut self, who: i32, idx: i32) -> bool {
        if self.a[49] != 0 {
            rd(&self.p[Self::tw(who, idx)], 0x2b8) & 0x20 != 0
        } else {
            self.log
                .push(format!("s17:p{}+0x0", Self::unit(who, idx)));
            self.a[4 + 17] != 0
        }
    }
    fn unit_is_supply(&mut self, who: i32, idx: i32) -> bool {
        if self.a[50] != 0 {
            rd(&self.p[Self::tw(who, idx)], 0x2b8) & 0x40 != 0
        } else {
            self.log
                .push(format!("s18:p{}+0x0", Self::unit(who, idx)));
            self.a[4 + 18] != 0
        }
    }
    fn unit_is_caravan(&mut self, who: i32, idx: i32) -> bool {
        if self.a[51] != 0 {
            self.log.push(format!("s21:p{}+0x0", Self::tw(who, idx)));
            self.a[4 + 21] != 0
        } else {
            self.log
                .push(format!("s19:p{}+0x0", Self::unit(who, idx)));
            self.a[4 + 19] != 0
        }
    }
    fn patch_version(&mut self) -> i32 {
        self.log.push("s14:c1+0x0".into());
        self.a[4 + 14]
    }
    fn game_bytes(&mut self) -> (i32, i32) {
        (i32::from(self.c[1][0x821]), i32::from(self.c[1][0x822]))
    }
    fn scenario_plunder(&mut self) -> bool {
        self.c[9][0] != 0
    }
    fn rand(&mut self, lo: i32, hi: i32) -> i32 {
        self.log.push(format!("s12:c7+0x0,{lo},{hi}"));
        let r = self.a[36 + self.n12.min(5)];
        self.n12 += 1;
        r
    }
    fn console_who(&mut self) -> i32 {
        rd(&self.c[2], 0x298)
    }
    fn str_assign_loc(&mut self, s: Str, off: i32) {
        self.log.push(format!(
            "s5:{},{}",
            Self::s(s),
            self.loc().wrapping_add(off)
        ));
    }
    fn str_cat_loc(&mut self, s: Str, off: i32) {
        self.log.push(format!(
            "s3:{},{}",
            Self::s(s),
            self.loc().wrapping_add(off)
        ));
    }
    fn str_cat_int(&mut self, s: Str, n: i32) {
        self.log.push(format!("s4:{},{n}", Self::s(s)));
    }
    fn str_cat_good(&mut self, s: Str, good: i32) {
        let name = rd(&self.p[25], 4 * good as usize).wrapping_add(0x74);
        self.log.push(format!("s3:{},{name}", Self::s(s)));
    }
    fn str_cat_str(&mut self, dst: Str, src: Str) {
        self.log
            .push(format!("s3:{},{}", Self::s(dst), Self::s(src)));
    }
    fn itow(&mut self, n: i32) {
        self.log.push(format!("s15:sp-0x70,{n},c10+0x0,1024,10"));
    }
    fn buff_len(&mut self) -> i32 {
        // The table ends in zeros: the window's end is a terminator.
        let b = &self.c[10];
        (0..b.len() / 2)
            .find(|&i| b[2 * i] == 0 && b[2 * i + 1] == 0)
            .unwrap_or(b.len() / 2) as i32
    }
    fn str_reinit(&mut self, s: Str, len: i32) -> i32 {
        self.log.push(format!("s7:{},{len},-1", Self::s(s)));
        self.a[4 + 7]
    }
    fn str_copy_buff(&mut self, _s: Str) {
        unreachable!("reinit answers non-zero in the sweep");
    }
    fn str_close(&mut self, s: Str) {
        self.log.push(format!("s6:{}", Self::s(s)));
    }
    fn pop_bubble(&mut self) -> usize {
        28
    }
    fn bubble_text(&mut self, b: usize, s: Str) {
        self.log.push(format!("s5:p{b}+0x4,{}", Self::s(s)));
    }
    fn bubble_who(&mut self, b: usize, who: i32) {
        self.p[b][0x1a] = who as u8;
    }
    fn neon(&mut self, who: i32) -> i32 {
        self.log.push(format!("s9:{},sp-0x48", self.tc(who)));
        self.a[4 + 9]
    }
    fn add_message(&mut self, s: Str, b: usize, color: i32, who: i32) {
        let (x, y) = self.this_coord();
        let tc = self.tc(who);
        self.log.push(format!(
            "s10:c3+0x0,{},{x},{y},0,{color},1,{tc},0,p{b}+0x0,{tc}",
            Self::s(s)
        ));
    }
    fn play(&mut self, sound: i32) {
        self.log.push(format!("s11:c3+0x0,{sound}"));
    }
}
let mut v = V {
    a,
    m,
    c,
    p,
    log: &mut log,
    n0: 0,
    n2: 0,
    n12: 0,
    n23: 0,
};
plunder_steps(&mut v, a[0], a[1], a[2], a[3]);
// The recorder prints `-` for a row that called nothing.
if log.is_empty() {
    log.push("-".into());
}
0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "plunder — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "plunder — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "plunder — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[12 + i], "plunder — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::plunder@00605b00: {n} rows agree");
}
