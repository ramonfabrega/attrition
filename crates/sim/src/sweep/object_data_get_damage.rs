//! `ObjectData::get_damage@00644130` under the emulator against the port (item 3356; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::get_damage@00644130`, run by `tools/emu/sweep_object_data_get_damage.py` under unicorn
/// (tools/emu): 96 random bytes of object, 6 argument(s), `p` windows in order: p0 argument 5, p1..p6 sparse table 0 (2 rows × 3 slice(s), row-major), p7..p43 the place(s), in flag order, p44..p52 the stubs' answer window(s); the global cell 0x00e85ddc points at 2192 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c061e4 points at 3072 random bytes; the global cell 0x00c061e8 points at 2096 random bytes; the global cell 0x00c061d4 points at 96 random bytes; 2112 random bytes at 0x00c06afc; argument 5 names 16 random bytes (null a third of the rows); callee 0x006db810 (1 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x0062dd90 (0 args) stubbed; callee 0x006469f0 (0 args) stubbed; callee 0x006da000 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0x124 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x184 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; the word at `0xc0ab84` points at 8 random bytes less `0` (placed); the word at `p7+0` points at 64 random bytes less `0` (placed); the word at `p7+4` points at 64 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 8 random bytes less `0` (placed); the word at `p10+0` points at 64 random bytes less `0` (placed); the word at `p10+4` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p13+0` points at 256 random bytes less `0` (placed); the word at `p13+4` points at 256 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 8 random bytes less `0` (placed); the word at `p16+0` points at 256 random bytes less `0` (placed); the word at `p16+4` points at 256 random bytes less `0` (placed); the word at `0xc0aaf4` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 32 random bytes less `0` (placed); the word at `p19+4` points at 32 random bytes less `0` (placed); the word at `p19+8` points at 32 random bytes less `0` (placed); the word at `p19+12` points at 32 random bytes less `0` (placed); the word at `c5+0x10` points at 8 random bytes less `0` (placed); the word at `p24+0` points at 128 random bytes less `0` (placed); the word at `p24+4` points at 128 random bytes less `0` (placed); the word at `c5+0x2c` points at 8 random bytes less `0` (placed); the word at `p27+0` points at 128 random bytes less `0` (placed); the word at `p27+4` points at 128 random bytes less `0` (placed); the word at `c0+0x888` points at 64 random bytes less `0` (placed); the word at `c2+0x134` points at 2560 random bytes less `0` (placed); the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `p3+0` points at 256 random bytes less `0` (placed); the word at `p6+0` points at 256 random bytes less `0` (placed); the word at `p8+0x18` points at 784 random bytes less `0` (placed); the word at `p9+0x18` points at 784 random bytes less `0` (placed); the word at `p11+0x18` points at 784 random bytes less `0` (placed); the word at `p12+0x18` points at 784 random bytes less `0` (placed); the word at `p14+0x18` points at 784 random bytes less `0` (placed); the word at `p15+0x18` points at 784 random bytes less `0` (placed); the word at `p17+0x18` points at 784 random bytes less `0` (placed); the word at `p18+0x18` points at 784 random bytes less `0` (placed); the word at `m+0x30` points at 512 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [-2, -1, 0, 1, 2] (90%); stub 3 answered from [4, 4, 0, 1, 2, 3] (90%); stub 4 answered from [0, 10, 50, 100, 500, 1000, -5, 3000] (90%); stub 5 answered from [0, 1, 2, 0, 1] (90%); stub 6 answered from [1, 1, 1, 0] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [0, 0, 1] (90%); stub 15 answered from [0, 0, 1] (90%); stub 16 answered from [0, 1, 1] (90%); stub 17 answered from [0, 1, 2, 3, 5, -1] (90%); stub 19 answered from [0, 1] (90%); stub 20 answered from [0, 10, 50, 100, 500, 1000, -3, 3000] (90%); stub 21 answered from [0, 1, 2, 5, 10, 50, -3, 200] (90%); stub 22 answered from [0, 1] (90%); stub 23 answered from [0, 1] (90%); stub 24 answered from [0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 9 answers a window of its own to each call, in call order (128 bytes, ...); stub 10 answers a window of its own to each call, in call order (64 bytes, ...); the word at `p44+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p45+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p46+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p47+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p48+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p49+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p50+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p44+0x18` laid as the address of `p43` every row; the word at `p45+0x18` laid as the address of `p43` every row; the word at `p46+0x18` laid as the address of `p43` every row; the word at `p47+0x18` laid as the address of `p43` every row; the word at `p48+0x18` laid as the address of `p43` every row; the word at `p49+0x18` laid as the address of `p43` every row; the word at `p50+0x18` laid as the address of `p43` every row; stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 6 answers 17 values in call order, printed after the answers; stub 7 answers 2 values in call order, printed after the answers; stub 8 answers 9 values in call order, printed after the answers; stub 11 answers 3 values in call order, printed after the answers; stub 12 answers 7 values in call order, printed after the answers; stub 13 answers 6 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 19 answers 5 values in call order, printed after the answers; stub 20 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `0xc061e0`, slices +0x59de:0x4, +0x6c5c:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_get_damage.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Row: `a[0]` and `a[1]` the target's index and player, `a[2]` the angle, `a[3]` the splash flag,
            // `a[4]` the overkill flag, `a[5]` the out-pointer (`p0` its 16 bytes, null a third of the rows);
            // `a[6 + i]` stub `i`'s answer; then the seq values in flag order (stub 0 `has_preq` 2, 1
            // `has_tribe_bonus` 2, 6 `is_unit` 17, 7 `is_building` 2, 8 `is_build` 9, 11 `is_active` 3, 12
            // the type's `is` 7, 13 the object's `is` 6, 14 `is_supply` 2, 19 `is_siege` 5, 20 `attack` 2);
            // then `a[89]` whether slot `+0xb8` held `ObjectData::is` (the inline arm) and `a[90]` whether
            // slot `+0xc` held `TypeData::is_unit_type`.
            // Stubs: 0 `LeaderData::has_preq`, 1 `has_tribe_bonus`, 2 `has_general`, 3 `BuildData::get_shot`,
            // 4 `ObjectData::attack`, 5 `LeaderData::get_target`, then the slots `+0x18` is_unit, `+0x1c`
            // is_building, `+0x20` is_build, `+0x3c` get_build, `+0x40` get_wall, `+0x4c` is_active, `+0x60`
            // the type's `is`, `+0xb8` the object's `is`, `+0xcc` is_supply, `+0xd0` is_caravan, `+0xd8`
            // is_moving, `+0xe4` get_captain, `+0xec` (unrecorded), `+0x10c` is_siege, `+0x120` attack,
            // `+0x124` armor, `+0x130` max_range, `+0x184` in_unfriendly_territory, `+0xc`, `+0x14` (6 to 25).
            // Windows: `m` the attacker, `c0` the types list cell (`p30` its object), `c1` `div_3_table`,
            // `c2` the world (`p31` its tiles), `c3` the constants, `c4` the game, `c5` the cities cell,
            // `c6` the combat table; `p1 + 3r ..` leader `r`'s `+0x59de`, `+0x6c5c` and `+0x6eb8` (`p33 + r`
            // its record), `p8 p9 p11 p12` the objects of players 0 and 1 (`p35..p38` their types), `p14 p15
            // p17 p18` the units (`p39..p42` their types), `p20..p23` the object types, `p25 p26 p28 p29` the
            // cities, `p32` the attacker's type, `p43` the building records' type, `p44..p50` the seven
            // `get_build` answers and `p51 p52` the two `get_wall` answers.
            //
            // **The port is the crate's own steps**, `crate::damage::get_damage_steps`, over a view that
            // reads the emulated original's records; the recorded calls (`log`) hold which object each
            // question went to, in order.
            use crate::damage::{DamageFacts, Rule, get_damage_steps};
            use crate::movement::Angle;

            const S_HAS_PREQ: usize = 0;
            const S_TRIBE: usize = 1;
            const S_GENERAL: usize = 2;
            const S_SHOT: usize = 3;
            const S_ATTACK: usize = 4;
            const S_TARGET: usize = 5;
            const S_UNIT: usize = 6;
            const S_BUILDING: usize = 7;
            const S_BUILD: usize = 8;
            const S_GET_BUILD: usize = 9;
            const S_GET_WALL: usize = 10;
            const S_ACTIVE: usize = 11;
            const S_TYPE_IS: usize = 12;
            const S_IS: usize = 13;
            const S_SUPPLY: usize = 14;
            const S_CARAVAN: usize = 15;
            const S_MOVING: usize = 16;
            const S_CAPTAIN: usize = 17;
            const S_SIEGE: usize = 19;
            const S_ATTACK_SLOT: usize = 20;
            const S_ARMOR: usize = 21;
            const S_RANGED: usize = 22;
            const S_UNFRIENDLY: usize = 23;
            const S_IS_UNIT_TYPE: usize = 24;
            const S_G14: usize = 25;

            const O_WIN: [usize; 4] = [8, 9, 11, 12];
            const O_TYPE: [usize; 4] = [35, 36, 37, 38];
            const U_WIN: [usize; 4] = [14, 15, 17, 18];
            const U_TYPE: [usize; 4] = [39, 40, 41, 42];
            const CITY: [usize; 4] = [25, 26, 28, 29];
            const BUILD_VFTABLE: i32 = 0x00b4_2174;

            /// A seq stub's base in the row and its length.
            fn seq_of(idx: usize) -> Option<(usize, usize)> {
                Some(match idx {
                    0 => (32, 2),
                    1 => (34, 2),
                    6 => (36, 17),
                    7 => (53, 2),
                    8 => (55, 9),
                    11 => (64, 3),
                    12 => (67, 7),
                    13 => (74, 6),
                    14 => (80, 2),
                    19 => (82, 5),
                    20 => (87, 2),
                    _ => return None,
                })
            }

            struct V<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
                cur: [usize; 26],
                n_build: usize,
                n_wall: usize,
            }

            impl V<'_> {
                fn who(&self) -> usize {
                    usize::from(self.m[9])
                }
                fn id(&self) -> usize {
                    usize::from(u16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                }
                fn att_k(&self) -> usize {
                    self.who() * 2 + self.id()
                }
                fn tgt_k(&self) -> usize {
                    self.a[1] as usize * 2 + self.a[0] as usize
                }
                fn word(&self, w: usize, off: usize) -> i32 {
                    rd(&self.p[w], off)
                }
                fn short(&self, w: usize, off: usize) -> i32 {
                    i32::from(i16::from_le_bytes([self.p[w][off], self.p[w][off + 1]]))
                }
                fn cw(&self, w: usize, off: usize) -> i32 {
                    rd(&self.c[w], off)
                }
                /// The stub's next answer: a seq stub's next value (the last repeated), else its one.
                fn answer(&mut self, idx: usize) -> i32 {
                    match seq_of(idx) {
                        Some((base, n)) => {
                            let v = self.a[base + self.cur[idx].min(n - 1)];
                            self.cur[idx] += 1;
                            v
                        }
                        None => self.a[6 + idx],
                    }
                }
                fn tok(w: usize) -> String {
                    format!("p{w}+0x0")
                }
                fn call(&mut self, idx: usize, recv: &str, args: &[i32]) -> i32 {
                    let mut t = format!("s{idx}:{recv}");
                    for x in args {
                        t.push_str(&format!(",{x}"));
                    }
                    self.log.push(t);
                    self.answer(idx)
                }
                fn leader(&mut self, idx: usize, row: usize, args: &[i32]) -> i32 {
                    self.call(idx, &format!("r{row}+0x0"), args)
                }
                /// `ObjectData::is`: slot `+0xb8` of the object, or, when it holds the base, the type's slot `+0x60`.
                fn is(&mut self, obj: &str, ty: usize, idx: i32, strict: i32) -> bool {
                    if self.a[89] != 0 {
                        self.call(S_TYPE_IS, &Self::tok(ty), &[idx, strict]) != 0
                    } else {
                        self.call(S_IS, obj, &[idx, strict]) != 0
                    }
                }
                fn tile(&self) -> usize {
                    let t = O_WIN[self.tgt_k()];
                    let y = ((self.word(t, 0x14) ^ 0x63637) >> 8) as usize;
                    let x = ((self.word(t, 0x10) ^ 0x63637) >> 8) as usize;
                    let idx = self
                        .cw(1, 4 * y)
                        .wrapping_mul(self.cw(2, 0))
                        .wrapping_add(self.cw(1, 4 * x));
                    idx as usize * 0x1c
                }
                fn get_build(&mut self, recv: &str) -> usize {
                    self.call(S_GET_BUILD, recv, &[]);
                    let k = self.n_build.min(6);
                    self.n_build += 1;
                    44 + k
                }
                fn get_wall(&mut self, recv: &str) -> usize {
                    self.call(S_GET_WALL, recv, &[]);
                    let k = self.n_wall.min(1);
                    self.n_wall += 1;
                    51 + k
                }
                /// `Build::vftable` at the record's start: its flag byte, else its slot `+0x4c`.
                fn active(&mut self, rec: usize) -> bool {
                    if rd(&self.p[rec], 0) == BUILD_VFTABLE {
                        self.p[rec][8] & 4 != 0
                    } else {
                        self.call(S_ACTIVE, &Self::tok(rec), &[]) != 0
                    }
                }
            }

            impl DamageFacts for V<'_> {
                fn rule(&mut self, r: Rule) -> i32 {
                    let off = match r {
                        Rule::RedFortAirDefense => 0x4c4,
                        Rule::WellingtonSiegeAttack => 0xbbc,
                        Rule::JapaneseDamage => 0x794,
                        Rule::RiverModifier => 0x68,
                        Rule::FlankBonus => 0x4c,
                        Rule::CavalryFlankBonus => 0x50,
                        Rule::VehicleFlankBonus => 0x54,
                        Rule::RockyModifier => 0x58,
                        Rule::OverkillFrames => 0x5c,
                        Rule::OverkillDamage => 0x60,
                        Rule::EntrenchmentModifier => 0x64,
                        Rule::AntipaterEntrenchBonus => 0xb98,
                        Rule::HeightIncrement => 0x44,
                        Rule::HeightBonus => 0x48,
                        Rule::RussianCossackDamage => 0x76c,
                        Rule::SuperImmune => 0x558,
                        Rule::RecaptureCityModifier => 0x6c,
                    };
                    self.cw(3, off)
                }
                fn pct(&mut self) -> i32 {
                    let a = self.word(O_TYPE[self.att_k()], 4);
                    let t = self.word(O_TYPE[self.tgt_k()], 4);
                    let at = 2 * (a * 0x1ed + t) as usize;
                    i32::from(i16::from_le_bytes([self.c[6][at], self.c[6][at + 1]]))
                }
                fn target_armor(&mut self) -> i32 {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_ARMOR, &t, &[])
                }
                fn attacker_attack(&mut self) -> i32 {
                    self.call(S_ATTACK_SLOT, "m+0x0", &[])
                }
                fn attacker_mask(&mut self) -> u32 {
                    self.word(32, 0x1e4) as u32
                }
                fn target_mask(&mut self) -> u32 {
                    self.word(O_TYPE[self.tgt_k()], 0x1e4) as u32
                }
                fn attacker_fort_or_city(&mut self) -> bool {
                    let idx = self.word(32, 4);
                    idx == 0x1bc || idx == 0x1bb || self.m[8] & 0x20 != 0
                }
                fn gunpowder(&mut self) -> bool {
                    let who = self.who();
                    let g = self.word(30, 4);
                    let is_unit_type = if self.a[90] != 0 {
                        (0x32..0x19e).contains(&g)
                    } else {
                        self.call(S_IS_UNIT_TYPE, &Self::tok(30), &[]) != 0
                    };
                    if is_unit_type && self.leader(S_HAS_PREQ, who, &[0x222]) == 0 {
                        return false;
                    }
                    if self.call(S_G14, &Self::tok(30), &[]) == 0 {
                        (self.p[2 + 3 * who][0] >> 2) & 1 != 0
                    } else {
                        self.leader(S_HAS_PREQ, who, &[0x222]) != 0
                    }
                }
                fn attacker_obj_is_build(&mut self) -> bool {
                    let o = Self::tok(O_WIN[self.att_k()]);
                    self.call(S_BUILD, &o, &[]) != 0
                }
                fn attacker_is_build(&mut self) -> bool {
                    self.call(S_BUILD, "m+0x0", &[]) != 0
                }
                fn target_is_unit(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_UNIT, &t, &[]) != 0
                }
                fn target_packs(&mut self) -> bool {
                    self.p[U_TYPE[self.tgt_k()]][0x2b8] & 4 != 0
                }
                fn target_packed(&mut self) -> bool {
                    self.word(U_WIN[self.tgt_k()], 0x68) & 0x80000 != 0
                }
                fn target_attacks(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_ATTACK_SLOT, &t, &[]) != 0
                }
                fn target_is_moving(&mut self) -> bool {
                    let t = Self::tok(U_WIN[self.tgt_k()]);
                    self.call(S_MOVING, &t, &[]) != 0
                }
                fn target_type_siege(&mut self) -> bool {
                    let t = Self::tok(O_TYPE[self.tgt_k()]);
                    self.call(S_SIEGE, &t, &[]) != 0
                }
                fn target_is_supply(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_SUPPLY, &t, &[]) != 0
                }
                fn target_is_caravan(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_CARAVAN, &t, &[]) != 0
                }
                fn target_is_build(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_BUILD, &t, &[]) != 0
                }
                fn target_build_active(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    let rec = self.get_build(&t);
                    self.active(rec)
                }
                fn attacker_citizen(&mut self) -> bool {
                    (0x32..=0x35).contains(&self.word(32, 4))
                }
                fn attacker_militia(&mut self) -> bool {
                    self.is("m+0x0", 32, 0x42, 0)
                }
                fn tile_owned_by_attacker(&mut self) -> bool {
                    let at = self.tile() + 0xf;
                    i32::from(self.p[31][at] as i8) == self.who() as i32
                }
                fn attacker_v2rocket(&mut self) -> bool {
                    self.is("m+0x0", 32, 0x139, 0)
                }
                fn attacker_is_unit(&mut self) -> bool {
                    self.call(S_UNIT, "m+0x0", &[]) != 0
                }
                fn attacker_domain(&mut self) -> i32 {
                    self.word(32, 0x218)
                }
                fn target_domain(&mut self) -> i32 {
                    self.word(O_TYPE[self.tgt_k()], 0x218)
                }
                fn target_redfort(&mut self) -> bool {
                    let k = self.tgt_k();
                    self.is(&Self::tok(O_WIN[k]), O_TYPE[k], 0x216, 1)
                }
                fn attacker_is_building(&mut self) -> bool {
                    self.call(S_BUILDING, "m+0x0", &[]) != 0
                }
                fn attacker_shot_redoubt(&mut self) -> bool {
                    let o = Self::tok(O_WIN[self.att_k()]);
                    let rec = self.get_build(&o);
                    self.call(S_SHOT, &Self::tok(rec), &[]) == 4
                }
                fn wellington_aura(&mut self) -> bool {
                    let who = self.who();
                    if self.short(1 + 3 * who, 0) == 0 {
                        return false;
                    }
                    let u = Self::tok(U_WIN[self.att_k()]);
                    self.call(S_GENERAL, &u, &[0, 0x170]) >= 0
                }
                fn target_factory_made(&mut self) -> bool {
                    let idx = self.word(O_TYPE[self.tgt_k()], 0x40);
                    if idx < 0 {
                        return false;
                    }
                    self.call(S_TYPE_IS, &Self::tok(20 + idx as usize), &[0x1af, 0]) != 0
                }
                fn attacker_obj_is_unit(&mut self) -> bool {
                    let o = Self::tok(O_WIN[self.att_k()]);
                    self.call(S_UNIT, &o, &[]) != 0
                }
                fn attacker_barracks_made(&mut self) -> bool {
                    self.word(U_TYPE[self.att_k()], 0x40) == 0x1ab
                }
                fn japanese_level(&mut self) -> Option<i32> {
                    let who = self.who();
                    if self.leader(S_TRIBE, who, &[0xf]) == 0 {
                        return None;
                    }
                    let ages = self.word(33 + who, 0xdc) ^ 0x62766;
                    let epoch = self.word(33 + who, 0xe8) ^ 0x63187;
                    Some(ages.min(epoch))
                }
                fn target_unit_mask_400000(&mut self) -> bool {
                    self.word(U_WIN[self.tgt_k()], 0x68) & 0x400000 != 0
                }
                fn target_unit_domain(&mut self) -> i32 {
                    self.word(U_TYPE[self.tgt_k()], 0x218)
                }
                fn team_untargeted(&mut self) -> bool {
                    if self.c[4][0x24] != 2 {
                        return false;
                    }
                    let row = self.a[1] as usize;
                    self.leader(S_TARGET, row, &[]) as u32 != u32::from(self.m[9])
                }
                fn target_uber_size(&mut self) -> i32 {
                    self.word(U_TYPE[self.tgt_k()], 0x308)
                }
                fn splash_percent(&mut self) -> i32 {
                    self.word(O_TYPE[self.att_k()], 0x204)
                }
                fn target_unit_type_siege(&mut self) -> bool {
                    let t = Self::tok(U_TYPE[self.tgt_k()]);
                    self.call(S_SIEGE, &t, &[]) != 0
                }
                fn target_bark(&mut self) -> bool {
                    let k = self.tgt_k();
                    self.is(&Self::tok(O_WIN[k]), O_TYPE[k], 0x143, 0)
                }
                fn target_unit_z(&mut self) -> i32 {
                    self.word(U_WIN[self.tgt_k()], 0xc) ^ 0x63637
                }
                fn target_decoy(&mut self) -> bool {
                    self.p[U_WIN[self.tgt_k()]][0x68] & 1 != 0
                }
                fn attack_stat(&mut self) -> i32 {
                    self.call(S_ATTACK, "m+0x0", &[])
                }
                fn target_is_building(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.call(S_BUILDING, &t, &[]) != 0
                }
                fn target_unfriendly(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    let wall = self.get_wall(&t);
                    self.call(S_UNFRIENDLY, &Self::tok(wall), &[]) != 0 && self.c[4][0x821] & 2 == 0
                }
                fn target_wall_is_build(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    let wall = self.get_wall(&t);
                    self.call(S_BUILD, &Self::tok(wall), &[]) != 0
                }
                fn target_type_attack_zero(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    self.get_build(&t);
                    // The record's `+0x18` is laid at `p43` every row (`--alias`).
                    self.word(43, 0x1e8) == 0
                }
                fn flank_level(&mut self, angle: i32) -> i32 {
                    let facing = self.word(U_WIN[self.tgt_k()], 0x50);
                    crate::combat::flank_level(Angle(facing), Angle(angle)).unwrap_or(0)
                }
                fn attacker_is_ranged(&mut self) -> bool {
                    self.call(S_RANGED, "m+0x0", &[]) != 0
                }
                fn frame(&mut self) -> i32 {
                    self.cw(4, 0x550)
                }
                fn target_damage_frame(&mut self) -> i32 {
                    self.word(U_WIN[self.tgt_k()], 0x4c)
                }
                fn attacker_captain(&mut self) -> i32 {
                    self.call(S_CAPTAIN, "m+0x0", &[])
                }
                fn target_damage_o(&mut self) -> i32 {
                    self.short(U_WIN[self.tgt_k()], 0xa4)
                }
                fn target_catapult(&mut self) -> bool {
                    let k = self.tgt_k();
                    self.is(&Self::tok(O_WIN[k]), O_TYPE[k], 0x109, 0)
                }
                fn attacker_type_siege(&mut self) -> bool {
                    self.call(S_SIEGE, &Self::tok(32), &[]) != 0
                }
                fn target_rocky(&mut self) -> bool {
                    self.p[31][self.tile()] & 8 != 0
                }
                fn attacker_z(&mut self) -> i32 {
                    rd(self.m, 0xc) ^ 0x63637
                }
                fn target_z(&mut self) -> i32 {
                    self.word(O_WIN[self.tgt_k()], 0xc) ^ 0x63637
                }
                fn attacker_obj_type_siege(&mut self) -> bool {
                    let t = Self::tok(O_TYPE[self.att_k()]);
                    self.call(S_SIEGE, &t, &[]) != 0
                }
                fn target_entrenched(&mut self) -> bool {
                    self.word(U_WIN[self.tgt_k()], 0x68) & 0x2000000 != 0
                }
                fn attacker_obj_flamethrower(&mut self) -> bool {
                    let k = self.att_k();
                    self.is(&Self::tok(O_WIN[k]), O_TYPE[k], 0x83, 0)
                }
                fn trench_facing(&mut self) -> i32 {
                    self.word(U_WIN[self.tgt_k()], 0x5c)
                }
                fn antipater_entrenched(&mut self) -> bool {
                    self.word(U_WIN[self.tgt_k()], 0x6c) & 0x1000 != 0
                }
                fn russian(&mut self) -> bool {
                    let who = self.who();
                    self.leader(S_TRIBE, who, &[0xd]) != 0
                }
                fn attacker_stable_made(&mut self) -> bool {
                    self.word(32, 0x40) == 0x1ac
                }
                fn target_type_supercollider(&mut self) -> bool {
                    self.word(O_TYPE[self.tgt_k()], 4) == 0x21d
                }
                fn recapturable_city(&mut self) -> bool {
                    let t = Self::tok(O_WIN[self.tgt_k()]);
                    let rec = self.get_build(&t);
                    if self.p[rec][8] & 0x20 == 0 {
                        return false;
                    }
                    let rec = self.get_build(&t);
                    let city = self.short(rec, 0x72);
                    if city < 0 {
                        return false;
                    }
                    let owner =
                        i32::from(self.p[CITY[self.a[1] as usize * 2 + city as usize]][0x5f] as i8);
                    owner == self.who() as i32
                }
            }

            let mut dtype = 0;
            let r = {
                let mut v = V {
                    a,
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    cur: [0; 26],
                    n_build: 0,
                    n_wall: 0,
                };
                get_damage_steps(
                    &mut v,
                    a[2],
                    a[3] != 0,
                    a[4] != 0,
                    (a[5] != 0).then_some(&mut dtype),
                )
            };
            if a[5] != 0 {
                wr(&mut p[0], 0, dtype);
            }
            i64::from(r)
        };
        assert_eq!(got, want_eax, "get_damage — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_damage — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_damage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "get_damage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("ObjectData::get_damage@00644130: {n} rows agree");
}
