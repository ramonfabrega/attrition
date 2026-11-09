//! `ObjectData::attack@006469f0` under the emulator against the port (item 3387; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::attack@006469f0`, run by `tools/emu/sweep_obj.py` on its row `object_data_attack` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p16 the place(s), in flag order; the global cell 0x00c061e4 points at 3328 random bytes; callee 0x006e1370 (1 args) stubbed; callee 0x006db810 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc8 (0 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p1+0` points at 224 random bytes less `0` (placed); the word at `p2+0` points at 224 random bytes less `0` (placed); the word at `p3+0` points at 224 random bytes less `0` (placed); the word at `p4+0` points at 224 random bytes less `0` (placed); the word at `p5+0` points at 224 random bytes less `0` (placed); the word at `p6+0` points at 224 random bytes less `0` (placed); the word at `p7+0` points at 224 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `0xc061e0`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_attack") else {
        return;
    };
    let obj = super::Obj {
        name: "attack",
        full: "ObjectData::attack@006469f0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 2000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `ObjectData::attack`'s steps (`sim::combat::object_attack_view`) over the windows: the type record
        // is `p[8]` (attack `+0x1e8`, id `+4`), the constants `c[0]` (dutch `+0x8b8`, caravan `+0xcd4`),
        // the owner `m[9]` (age count `p[9 + who]` `+0xdc` xor 0x62766); the stubs answer `a[0..6]`
        // (has_tribe_bonus, has_preq, slots 0x18, 0xd0, 0xcc, 0xc8) and record their calls.
        struct V<'a> {
            a: &'a [i32],
            m: &'a [u8],
            c: &'a [Vec<u8>],
            p: &'a [Vec<u8>],
            log: &'a mut Vec<String>,
        }
        impl V<'_> {
            fn who(&self) -> usize {
                usize::from(self.m[9])
            }
        }
        impl sim::combat::object_attack_view::ObjectAttackView for V<'_> {
            fn type_attack(&mut self) -> i32 {
                rd(&self.p[8], 0x1e8)
            }
            fn is_caravan(&mut self) -> bool {
                self.log.push("s3:m+0x0".into());
                self.a[3] != 0
            }
            fn has_caravan_preq(&mut self) -> bool {
                let who = self.who();
                self.log.push(format!("s1:r{who}+0x0,698"));
                self.a[1] != 0
            }
            fn caravan_attack_bonus(&mut self) -> i32 {
                rd(&self.c[0], 0xcd4)
            }
            fn has_dutch_bonus(&mut self) -> bool {
                let who = self.who();
                self.log.push(format!("s0:r{who}+0x0,22"));
                self.a[0] != 0
            }
            fn dutch_attack_bonus(&mut self) -> i32 {
                rd(&self.c[0], 0x8b8)
            }
            fn is_unit(&mut self) -> bool {
                self.log.push("s2:m+0x0".into());
                self.a[2] != 0
            }
            fn type_index(&mut self) -> i32 {
                rd(&self.p[8], 4)
            }
            fn is_supply(&mut self) -> bool {
                self.log.push("s4:m+0x0".into());
                self.a[4] != 0
            }
            fn gov_hero_slot(&mut self) -> bool {
                self.log.push("s5:m+0x0".into());
                self.a[5] != 0
            }
            fn ages(&mut self) -> i32 {
                rd(&self.p[9 + self.who()], 0xdc) ^ 0x62766
            }
        }
        let mut v = V {
            a,
            m,
            c,
            p,
            log: &mut *log,
        };
        i64::from(sim::combat::object_attack_view::object_attack_steps(&mut v))
    });
}
