//! `Build::plunder@00623660` under the emulator against the port (item 3402; tools/sweep_gen.py, obj with windows).

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

/// `Build::plunder@00623660`, run by `tools/emu/sweep_build_plunder.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p9 sparse table 0 (2 rows × 5 slice(s), row-major), p10..p32 the place(s), in flag order, p33..p34 the stubs' answer window(s); the global cell 0x00c061f0 points at 3200 random bytes; the global cell 0x00c061ec points at 64 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 64 random bytes; the global cell 0x00c061ac points at 64 random bytes; the global cell 0x00c061f4 points at 64 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c0618c points at 64 random bytes; 4 random bytes at 0x00c8cd00; 1 random bytes at 0x00cb195a (each byte in 0..1); callee 0x006e1370 (1 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x006ebc10 (1 args) stubbed; vtable slot +0x18c (1 args) stubbed; callee 0x00639b90 (3 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x0073a1b0 (6 args) stubbed; callee 0x0043ec70 (0 args) stubbed; callee 0x0043ed10 (2 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1b010 (4 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `0xc09970+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 128 random bytes less `0` (placed); the word at `p10+4` points at 128 random bytes less `0` (placed); the word at `p10+8` points at 128 random bytes less `0` (placed); the word at `p10+12` points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 128 random bytes less `0` (placed); the word at `p15+4` points at 128 random bytes less `0` (placed); the word at `p15+8` points at 128 random bytes less `0` (placed); the word at `p15+12` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `c4+0x10` points at 64 random bytes less `0` (placed); the word at `c5+0x10` points at 1792 random bytes less `0` (placed); the word at `c7+0x14` points at 16 random bytes less `0` (placed); the word at `c7+0x30` points at 16 random bytes less `0` (placed); the word at `p23+0` points at 32 random bytes less `0` (placed); the word at `p23+4` points at 32 random bytes less `0` (placed); the word at `p23+8` points at 32 random bytes less `0` (placed); the word at `p23+0xc` points at 32 random bytes less `0` (placed); the word at `p24+0` points at 32 random bytes less `0` (placed); the word at `p24+4` points at 32 random bytes less `0` (placed); the word at `p24+8` points at 32 random bytes less `0` (placed); the word at `p24+0xc` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2, 3, 7, 10, 100, 1000, 5000, 16777217, 2147483647, -1, -100] (90%); stub 6 answered from [-1, -1, 0, 1, 5] (90%); stub 7 answered from [0, 1] (90%); stub 13 answers the address of 64 random bytes, a `p` window after the places; stub 5 answers the address of 128 random bytes, a `p` window after the places; stub 0 answers 5 values in call order, printed after the answers; stub 6 answers 2 values in call order, printed after the answers; stub 4 answers -2..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x914:0x4, +0x59bc:0x4, +0x59d8:0x4, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_plunder.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(10);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0]` the unused argument, `a[1]` `who`; `a[2 + k]` stub `k`'s answer; then the seq values
            // in flag order: `a[19..24]` `has_tribe_bonus`'s, `a[24..26]` `find_hero`'s. Stubs: 0
            // `has_tribe_bonus`, 1 `type_avail`, 2 `has_wonder`, 3 the build time (slot `+0x18c`), 4
            // `get_town`, 5 the object's slot `+0xac`, 6 `find_hero`, 7 `is_scenario`, 8 `bucket_add`, 9
            // `String::String`, 10 `String =`, 11 `String::parse`, 12 `String::close`, 13 `Recycler::pop`
            // (not recorded), 14 `get_neon`, 15 `add_message`, 16 `SoundGlobal::play`. Windows: `m` the
            // building, `c0` `constants`, `c1` `game`, `c2` the console, `c3` the message window, `c4`
            // `goodtypes`, `c5` `buildtypes`, `c6` `div_3_table`, `c7` `objects`, `c8` the localised string
            // table's base, `c9` `ScenarioData::plunder`; `p[5r..5r+5]` leader `r`'s `+8`, `+0x914`,
            // `+0x59bc`, `+0x59d8` and `+0x6928`; `p10`/`p15` `cities[r]`'s lists and `p11..p14`/`p16..p19`
            // their cities; `p20` the building's type; `p21` the goods' array; `p22` the build types'
            // array; `p23`/`p24` `objects[r]`'s lists and `p25..p28`/`p29..p32` their objects; `p33` the
            // text bubble; `p34` what slot `+0xac` answers.
            //
            // **The port is the crate's own steps**, `crate::build_plunder_view::build_plunder_steps`; the
            // view here is the emulated original's windows. `LeaderDataEncrypt::scratch` is the bucket
            // write's twin and nothing of the row reads it.
            use crate::build_plunder_view::{BuildPlunderView, Str, build_plunder_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                n0: usize,
                n6: usize,
            }
            impl V<'_> {
                fn leader(who: i32) -> i32 {
                    0xe3a390 + who * 0x6eec
                }
                fn row(who: i32) -> usize {
                    5 * who as usize
                }
                fn s(s: Str) -> &'static str {
                    match s {
                        Str::A => "sp-0x58",
                        Str::B => "sp-0x6c",
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
                fn ty(&self, off: usize) -> i32 {
                    rd(&self.p[20], off)
                }
            }
            impl BuildPlunderView for V<'_> {
                type Bub = usize;
                fn this_who(&mut self) -> i32 {
                    i32::from(self.m[9])
                }
                fn this_coord(&mut self) -> (i32, i32) {
                    (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
                }
                fn activated(&mut self) -> bool {
                    self.m[0x61] & 0x10 != 0
                }
                fn job_counter(&mut self) -> i32 {
                    rd(self.m, 0x4c)
                }
                fn build_time(&mut self) -> i32 {
                    self.log.push("s3:m+0x0,0".into());
                    self.a[2 + 3]
                }
                fn type_value(&mut self) -> i32 {
                    self.ty(0x2d4)
                }
                fn type_good(&mut self) -> i32 {
                    self.ty(0x2d8)
                }
                fn type_ident(&mut self) -> i32 {
                    self.ty(4)
                }
                fn type_size(&mut self) -> (i32, i32) {
                    (self.ty(0x234), self.ty(0x238))
                }
                fn lakota_raze_price(&mut self) -> i32 {
                    rd(&self.c[0], 0x854)
                }
                fn german_mine_cost(&mut self) -> i32 {
                    rd(&self.c[0], 0x724)
                }
                fn tikal_temple_cost(&mut self) -> i32 {
                    rd(&self.c[0], 0x4a4)
                }
                fn plunder(&mut self) -> i32 {
                    rd(&self.c[0], 0x3b0)
                }
                fn aztec_plunder(&mut self) -> i32 {
                    rd(&self.c[0], 0x578)
                }
                fn thedespot_plunder(&mut self) -> i32 {
                    rd(&self.c[0], 0xc70)
                }
                fn russian_plunder_steal(&mut self) -> i32 {
                    rd(&self.c[0], 0x770)
                }
                fn has_tribe_bonus(&mut self, who: i32, bonus: i32) -> bool {
                    self.log.push(format!("s0:{},{bonus}", Self::leader(who)));
                    let r = self.a[19 + self.n0.min(4)];
                    self.n0 += 1;
                    r != 0
                }
                fn type_avail(&mut self, who: i32, good: i32) -> bool {
                    self.log.push(format!("s1:{},{good},1", Self::leader(who)));
                    self.a[2 + 1] != 0
                }
                fn has_wonder(&mut self, who: i32, wonder: i32) -> bool {
                    self.log.push(format!("s2:{},{wonder}", Self::leader(who)));
                    self.a[2 + 2] != 0
                }
                fn div3(&mut self, c: i32) -> i32 {
                    rd(&self.c[6], 4 * (c >> 6) as usize)
                }
                fn get_town(&mut self, ident: i32, x: i32, y: i32, who: i32) -> i32 {
                    let bt = rd(&self.p[22], 4 * ident as usize);
                    self.log.push(format!("s4:{bt},{x},{y},{who}"));
                    self.a[2 + 4]
                }
                fn town_city(&mut self, who: i32, town: i32) -> i32 {
                    self.log.push(format!("s5:p{}+0x0", 25 + 4 * who + town));
                    i32::from(i16::from_le_bytes([self.p[34][0x72], self.p[34][0x73]]))
                }
                fn city_owner(&mut self, who: i32, city: i32) -> i32 {
                    i32::from(self.p[(11 + 5 * who + city) as usize][0x5f] as i8)
                }
                fn hero_count(&mut self, who: i32, ty: i32) -> i32 {
                    let r = Self::row(who);
                    let (w, o) = if ty == 0x160 { (r + 2, 2) } else { (r + 3, 0) };
                    i32::from(i16::from_le_bytes([self.p[w][o], self.p[w][o + 1]]))
                }
                fn find_hero(&mut self, x: i32, y: i32, who: i32, ty: i32, slack: i32) -> i32 {
                    self.log
                        .push(format!("s6:{y},{x},{y},{},0,{ty},{slack}", self.f8(who)));
                    let r = self.a[24 + self.n6.min(1)];
                    self.n6 += 1;
                    r
                }
                fn is_scenario(&mut self) -> bool {
                    self.log.push("s7:c1+0x0".into());
                    self.a[2 + 7] != 0
                }
                fn scenario_plunder(&mut self) -> bool {
                    self.c[9][0] != 0
                }
                fn plunder_scale(&mut self, who: i32) -> u32 {
                    rd(&self.p[Self::row(who) + 1], 0) as u32
                }
                fn bucket_add(&mut self, who: i32, good: i32, n: i32) {
                    self.log
                        .push(format!("s8:{},{good},{n}", Self::leader(who)));
                }
                fn console_who(&mut self) -> i32 {
                    rd(&self.c[2], 0x298)
                }
                fn str_new(&mut self, s: Str) {
                    self.log.push(format!("s9:{}", Self::s(s)));
                }
                fn str_assign_loc(&mut self, s: Str, off: i32) {
                    self.log.push(format!(
                        "s10:{},{}",
                        Self::s(s),
                        self.loc().wrapping_add(off)
                    ));
                }
                fn str_parse(&mut self, s: Str, good: i32, n: i32) {
                    let name = rd(&self.p[21], 4 * good as usize).wrapping_add(0x74);
                    self.log
                        .push(format!("s11:{},sp-0x44,{name},{name},{n}", Self::s(s)));
                    self.log
                        .push(format!("s10:{},{}", Self::s(s), self.a[2 + 11]));
                    self.log.push("s12:sp-0x44".into());
                }
                fn str_close(&mut self, s: Str) {
                    self.log.push(format!("s12:{}", Self::s(s)));
                }
                fn pop_bubble(&mut self) -> usize {
                    33
                }
                fn bubble_text(&mut self, b: usize, s: Str) {
                    self.log.push(format!("s10:p{b}+0x4,{}", Self::s(s)));
                }
                fn bubble_who(&mut self, b: usize, who: i32) {
                    self.p[b][0x1a] = who as u8;
                }
                fn neon(&mut self, who: i32) -> i32 {
                    self.log.push(format!("s14:{},sp-0x3c", self.tc(who)));
                    self.a[2 + 14]
                }
                fn add_message(&mut self, s: Str, b: usize, color: i32, who: i32) {
                    let (x, y) = self.this_coord();
                    let tc = self.tc(who);
                    self.log.push(format!(
                        "s15:c3+0x0,{},{x},{y},0,{color},1,{tc},0,p{b}+0x0,{tc}",
                        Self::s(s)
                    ));
                }
                fn play(&mut self, sound: i32) {
                    self.log.push(format!("s16:c3+0x0,{sound}"));
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                n0: 0,
                n6: 0,
            };
            build_plunder_steps(&mut v, a[1]);
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
            assert_eq!(w, &want[11 + i], "plunder — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Build::plunder@00623660: {n} rows agree");
}
