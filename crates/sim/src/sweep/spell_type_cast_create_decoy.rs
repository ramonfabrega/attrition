//! `SpellType::cast_create_decoy@00674370` under the emulator against the port (item 3320; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_create_decoy@00674370`, run by `tools/emu/sweep_spell_type_cast_create_decoy.py` under unicorn
/// (tools/emu): 480 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p31 the place(s), in flag order; the global cell 0x00c0618c points at 384 random bytes; the global cell 0x00c061f0 points at 3072 random bytes; the global cell 0x00c06210 points at 704 random bytes; vtable slot +0xd8 (3 args) stubbed; callee 0x006e0830 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xe8 (0 args) stubbed; callee 0x00649d70 (0 args) stubbed; vtable slot +0x120 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; callee 0x00739e50 (0 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; callee 0x006f9f40 (1 args) stubbed; callee 0x006e0dd0 (3 args) stubbed; vtable slot +0x15c (1 args) stubbed; vtable slot +0x160 (0 args) stubbed; callee 0x006055c0 (0 args) stubbed; callee 0x006054c0 (0 args) stubbed; the word at `0xc0aec0+0x1c*a[1]` points at 24 random bytes less `0` (placed); the word at `p8+0` points at 160 random bytes less `0` (placed); the word at `p8+4` points at 160 random bytes less `0` (placed); the word at `p8+8` points at 160 random bytes less `0` (placed); the word at `p8+12` points at 160 random bytes less `0` (placed); the word at `p8+16` points at 160 random bytes less `0` (placed); the word at `p8+20` points at 160 random bytes less `0` (placed); the word at `0xc0a130+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 16 random bytes less `0` (placed); the word at `p15+4` points at 16 random bytes less `0` (placed); the word at `p15+8` points at 16 random bytes less `0` (placed); the word at `p15+12` points at 16 random bytes less `0` (placed); the word at `0xc09710+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 16 random bytes less `0` (placed); the word at `p20+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 16 random bytes less `0` (placed); the word at `p20+12` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[1]` points at 24 random bytes less `0` (placed); the word at `p9+0x18` points at 768 random bytes less `0` (placed); the word at `p10+0x18` points at 768 random bytes less `0` (placed); the word at `p11+0x18` points at 768 random bytes less `0` (placed); the word at `p12+0x18` points at 768 random bytes less `0` (placed); the word at `p13+0x18` points at 768 random bytes less `0` (placed); the word at `p14+0x18` points at 768 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; stub 1 answered from [-2, -1, -1, 0, 0, 1, 1, 2, 3, 3] (90%); stub 2 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 4 answered from [1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 6 answered from [1, 1, 1, 1, 1, 1, 0] (90%); stub 7 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 8 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 10 answered from [0, 0, 0, 0, 0, 0, 0, 0, 0, 1] (90%); the word at `p25+0` laid as the address of `p9` every row; the word at `p25+4` laid as the address of `p10` every row; the word at `p25+8` laid as the address of `p11` every row; the word at `p25+12` laid as the address of `p12` every row; the word at `p25+16` laid as the address of `p13` every row; the word at `p25+20` laid as the address of `p14` every row; stub 2 answers 12 values in call order, printed after the answers; stub 3 answers 12 values in call order, printed after the answers; stub 4 answers 14 values in call order, printed after the answers; stub 6 answers 7 values in call order, printed after the answers; stub 7 answers 7 values in call order, printed after the answers; stub 8 answers 7 values in call order, printed after the answers; stub 9 answers 7 values in call order, printed after the answers; stub 10 answers 7 values in call order, printed after the answers; stub 11 answers 7 values in call order, printed after the answers; stub 5 answers -1..3 every row; stub 9 answers 0..12 every row; stub 11 answers -1..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 10 writes a value in -300000..300000 through its argument 2, printed after the answers (`o` in a place); stub 10 writes a value in -300000..300000 through its argument 3, printed after the answers (`o` in a place); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x808:0x4, +0x93c:0x8 (`p` windows after the pointed buffers, before the places); figures p9..p14 link at +0x90 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_create_decoy.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**, `crate::cast_decoy::cast_create_decoy_steps`, which
                // `Sim::cast_create_decoy` runs over its `SimDecoy`; here the view is the emulated original's windows.
                //
                // `a`: the four stack arguments (`g`, `who`, two unused), then the eighteen stubs' single answers
                // (`a[4 + k]`: 1 `get_general_upgrade`, 5 `get_army`), the two words `find_nearby_spot` writes through
                // its arguments 2 and 3 (the spot, `a[22]`, `a[23]`), the call-order answers of stubs 2, 3, 4, 6, 7, 8,
                // 9, 10, 11 (12, 12, 14, 7, 7, 7, 7, 7, 7 values from `a[24]`), and the two vtable flags (`a[104]`:
                // slot +0xb8 laid as `ObjectData::is`, `a[105]`: slot +0xd0 as `UnitData::is_caravan`).
                // Stubs: 0 `unpay_cost` (this's slot +0xd8), 1 `get_general_upgrade`, 2 the unit's `is`, 3 the type's
                // `is` (+0x60), 4 `is_captain` (+0xe8), 5 `get_army`, 6 the unit's +0x120, 7 its +0xd0, 8 the type's
                // +0x130, 9 `get_radius`, 10 `find_nearby_spot`, 11 `init_unit`, 12 `Army::add_unit`, 13
                // `track_unit_type`, 14 and 15 the new unit's +0x15c and +0x160, 16 `update_speed`, 17 `update_armor`.
                // Windows: `c[0]` the objects singleton (the player's count `+0x15c + 4 * who`), `c[1]` the constants
                // (`+0xb80` Porus, `+0xbd8` Kutosov), `c[2]` the console; `p0..p7` the leaders table's slices (a row a
                // player: `+0x808`, then `+0x93c` and `+0x940`), `p9..p14` the six units (`p8` their array, `p25` the
                // objects' list, the same records), `p16..p19` the heroes, `p21..p24` the armies, `p26..p31` the six
                // units' type records.
                use crate::Pos;
                use crate::cast_decoy::{DecoyView, cast_create_decoy_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 12],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes(
                        w[o..o + 2].try_into().expect("two bytes"),
                    ))
                }
                impl V<'_> {
                    fn who(&self) -> usize {
                        self.a[1] as usize
                    }
                    fn unit(&self, o: i32) -> &[u8] {
                        &self.p[9 + o as usize]
                    }
                    /// Stub `k`'s next answer of `n` from `a[base]`, the last repeated.
                    fn seq(&mut self, k: usize, base: usize, n: usize) -> i32 {
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[base + i]
                    }
                    fn leader(&self) -> i32 {
                        14918544 + 0x6eec * self.a[1]
                    }
                }
                impl DecoyView for V<'_> {
                    fn general_upgrade(&mut self) -> i32 {
                        self.log.push(format!("s1:{}", self.leader()));
                        self.a[5]
                    }
                    fn is(&mut self, o: i32, ty: i32) -> bool {
                        if self.a[104] == 0 {
                            self.log.push(format!("s2:p{}+0x0,{ty},0", 9 + o));
                            self.seq(2, 24, 12) != 0
                        } else {
                            self.log.push(format!("s3:p{}+0x0,{ty},0", 26 + o));
                            self.seq(3, 36, 12) != 0
                        }
                    }
                    fn porus_decoys(&self) -> i32 {
                        rd32(&self.c[1], 0xb80)
                    }
                    fn kutosov_decoys(&self) -> i32 {
                        rd32(&self.c[1], 0xbd8)
                    }
                    fn count(&self) -> i32 {
                        rd32(&self.c[0], 0x15c + 4 * self.who())
                    }
                    fn nth(&self, k: i32) -> i32 {
                        k
                    }
                    fn alive(&self, o: i32) -> bool {
                        self.unit(o)[8] & 1 != 0
                    }
                    fn is_captain(&mut self, o: i32) -> bool {
                        self.log.push(format!("s4:p{}+0x0", 9 + o));
                        self.seq(4, 48, 14) != 0
                    }
                    fn is_decoy(&self, o: i32) -> bool {
                        rd32(self.unit(o), 0x68) & 1 != 0
                    }
                    fn is_computer(&self, o: i32) -> bool {
                        rd32(self.unit(o), 0x68) & 0x40000 != 0
                    }
                    fn army_of(&mut self, o: i32) -> i32 {
                        self.log.push(format!("s5:p{}+0x0", 9 + o));
                        self.a[9]
                    }
                    fn land(&self, o: i32) -> bool {
                        rd32(&self.p[26 + o as usize], 0x218) == 0
                    }
                    fn armed(&mut self, o: i32) -> bool {
                        self.log.push(format!("s6:p{}+0x0", 9 + o));
                        self.seq(6, 62, 7) != 0
                    }
                    fn type_index(&self, o: i32) -> i32 {
                        rd32(&self.p[26 + o as usize], 4)
                    }
                    fn is_caravan(&mut self, o: i32) -> bool {
                        if self.a[105] == 0 {
                            self.log.push(format!("s7:p{}+0x0", 9 + o));
                            self.seq(7, 69, 7) != 0
                        } else {
                            self.log.push(format!("s8:p{}+0x0", 26 + o));
                            self.seq(8, 76, 7) != 0
                        }
                    }
                    fn pos(&self, o: i32) -> Pos {
                        Pos::new(
                            rd32(self.unit(o), 0x10) ^ 0x63637,
                            rd32(self.unit(o), 0x14) ^ 0x63637,
                        )
                    }
                    fn hero_radius(&mut self, caster: i32) -> i32 {
                        let hero = rd16(self.unit(caster), 0x86);
                        self.log.push(format!("s9:p{}+0x0", 16 + hero));
                        self.seq(9, 83, 7)
                    }
                    fn find_spot(&mut self, caster: i32, _src: i32) -> Option<Pos> {
                        let at = self.pos(caster);
                        let (short, byte) = (rd16(self.unit(caster), 0xa), self.unit(caster)[9]);
                        self.log.push(format!(
                            "s10:p{}+0x0,{},{},sp-0x3c,sp-0x38,384,-1,0,0,3,{short},{byte},0,0,-1,0,-1",
                            26 + caster,
                            at.x,
                            at.y
                        ));
                        (self.seq(10, 90, 7) == 0).then(|| Pos::new(self.a[22], self.a[23]))
                    }
                    fn init_unit(&mut self, src: i32, at: Pos) -> i32 {
                        self.log.push(format!(
                            "s11:c0+0x0,{},{},{},{},-1,-1,-1",
                            self.a[1],
                            self.type_index(src),
                            at.x,
                            at.y
                        ));
                        self.seq(11, 97, 7)
                    }
                    fn army_add(&mut self, army: i32, o: i32) {
                        self.log.push(format!("s12:p{}+0x0,{o}", 21 + army));
                    }
                    fn type_pop(&self, o: i32) -> i32 {
                        rd32(&self.p[26 + o as usize], 0x2f0)
                    }
                    fn track_unit_type(&mut self, src: i32, o: i32) {
                        self.log.push(format!(
                            "s13:{},{},-1,{o}",
                            self.leader(),
                            self.type_index(src)
                        ));
                    }
                    fn leader_counts(&mut self, pop: i32) {
                        let who = self.who();
                        let w = &mut self.p[2 * who + 1];
                        let (n, v) = (rd32(w, 0), rd32(w, 4));
                        wr32(w, 4, v.wrapping_sub(pop));
                        wr32(w, 0, n.wrapping_sub(1));
                        let w = &mut self.p[2 * who];
                        let n = rd32(w, 0);
                        wr32(w, 0, n.wrapping_sub(1));
                    }
                    fn mark(&mut self, o: i32) {
                        let w = &mut self.p[9 + o as usize];
                        let f = rd32(w, 0x68);
                        wr32(w, 0x68, f | 1);
                        w[0x96..0x98].copy_from_slice(&0i16.to_le_bytes());
                    }
                    fn refresh(&mut self, o: i32) {
                        for k in [14, 15, 16, 17] {
                            let arg = if k == 14 { ",0" } else { "" };
                            self.log.push(format!("s{k}:p{}+0x0{arg}", 9 + o));
                        }
                    }
                    fn o_down(&self, o: i32) -> i32 {
                        rd16(self.unit(o), 0x90)
                    }
                    fn sound(&mut self) {}
                    fn unpay_cost(&mut self, caster: i32) {
                        self.log.push(format!("s0:m+0x0,{},{caster},-1", self.a[1]));
                    }
                    fn mana_burn(&self, o: i32) -> i16 {
                        rd16(self.unit(o), 0x96) as i16
                    }
                    fn set_mana_burn(&mut self, o: i32, v: i16) {
                        self.p[9 + o as usize][0x96..0x98].copy_from_slice(&v.to_le_bytes());
                    }
                    fn spell_mana(&self) -> Option<i32> {
                        Some(rd32(self.m, 0x1d0))
                    }
                }
                let g = a[0];
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    turn: [0; 12],
                };
                cast_create_decoy_steps(&mut v, g);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_create_decoy — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_create_decoy — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cast_create_decoy — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "cast_create_decoy — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3500, "only {n} rows");
    eprintln!("SpellType::cast_create_decoy@00674370: {n} rows agree");
}
