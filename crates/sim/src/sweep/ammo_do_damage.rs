//! `Ammo::do_damage@00678060` under the emulator against the port (item 3356; tools/sweep_gen.py, obj with windows).

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

/// `Ammo::do_damage@00678060`, run by `tools/emu/sweep_ammo_do_damage.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), `p` windows in order: p0..p11 sparse table 0 (3 rows × 4 slice(s), row-major), p12..p14 sparse table 1 (3 rows × 1 slice(s), row-major), p15..p41 the place(s), in flag order, p42..p43 the stubs' answer window(s); the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00cae5fc points at 704 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c06208 points at 16 random bytes; the global cell 0x00c06214 points at 16 random bytes; the global cell 0x00c06184 points at 16 random bytes; callee 0x00a3ab00 (10 args) stubbed; callee 0x009096e0 (1 args) stubbed; callee 0x00678f90 (0 args) stubbed; callee 0x00678d90 (1 args) stubbed; callee 0x0064a480 (8 args) stubbed; callee 0x006791a0 (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x006b53a0 (2 args) stubbed; callee 0x009072e0 (2 args) stubbed; callee 0x00679240 (0 args) stubbed; callee 0x0043f360 (2 args) stubbed; callee 0x006b4830 (2 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x007af660 (3 args) stubbed; callee 0x006625c0 (2 args) stubbed; callee 0x0092ba30 (6 args) stubbed; callee 0x006b4bb0 (4 args) stubbed; callee 0x007e9b70 (9 args) stubbed; callee 0x0092bc80 (0 args) stubbed; callee 0x0097f340 (3 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `c0+0x14+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p15+0` points at 64 random bytes less `0` (placed); the word at `p15+4` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p18+0` points at 64 random bytes less `0` (placed); the word at `p18+4` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p21+0` points at 64 random bytes less `0` (placed); the word at `p21+4` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*8` points at 8 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p27+0` points at 32 random bytes less `0` (placed); the word at `p27+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p30+0` points at 32 random bytes less `0` (placed); the word at `p30+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 8 random bytes less `0` (placed); the word at `p33+0` points at 32 random bytes less `0` (placed); the word at `p33+4` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 776 random bytes less `0` (placed); the word at `p17+0x18` points at 776 random bytes less `0` (placed); the word at `p22+0x18` points at 776 random bytes less `0` (placed); the word at `p23+0x18` points at 776 random bytes less `0` (placed); the word at `c2+0x134` points at 2688 random bytes less `0` (placed); the word at `c2+0x138` points at 1344 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [9, 0, 0, 1, 2, 3, 4, 5] (90%); stub 2 answered from [0, 0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 10 answered from [0, 1] (90%); stub 11 answered from [0, 1, 0, 0] (90%); stub 12 answered from [0, 1] (90%); stub 23 answered from [0, 1, 1] (90%); stub 24 answered from [0, 1] (90%); stub 25 answered from [0, 1] (90%); stub 26 answered from [0, 1] (90%); stub 29 answered from [0, 1] (90%); stub 30 answered from [0, 1] (90%); stub 27 answers a window of its own to each call, in call order (32 bytes, ...); the word at `p42+0x18` laid as the address of `p36` every row; the word at `p43+0x18` laid as the address of `p37` every row; the word at `p19+0x18` laid as the address of `p36` every row; the word at `p20+0x18` laid as the address of `p37` every row; the word at `p25+0x18` laid as the address of `p39` every row; the word at `p26+0x18` laid as the address of `p38` every row; the word at `p28+0x18` laid as the address of `p36` every row; the word at `p29+0x18` laid as the address of `p37` every row; the word at `p31+0x18` laid as the address of `p36` every row; the word at `p32+0x18` laid as the address of `p37` every row; the word at `p34+0x18` laid as the address of `p38` every row; the word at `p35+0x18` laid as the address of `p39` every row; stub 6 answers 2 values in call order, printed after the answers; stub 11 answers 8 values in call order, printed after the answers; stub 24 answers 8 values in call order, printed after the answers; stub 25 answers 8 values in call order, printed after the answers; stub 26 answers 8 values in call order, printed after the answers; stub 6 answers 0..65535 every row; stub 28 answers 0..9 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 7 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 7 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 11 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 11 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 7 writes a value in 0..2047 through its argument 0, printed after the answers (`o` in a place); stub 7 writes a value in 0..2047 through its argument 1, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); stub 3 writes 1 (4 bytes) at `m+0x4c` each call; stub 3 writes 1 (4 bytes) at `m+0x48` each call; 3 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x20, +0x7b4:0x10, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); 3 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_ammo_do_damage.py") else {
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
            // Row: `a[i]` stub `i`'s answer (stubs 0 to 30, in the line's order: 0 `Log::say`, 1
            // `get_gpiece_type`, 2 `hit_target`, 3 `check_hit`, 4 `Object::do_damage`, 5 `close`, 6
            // `Random::get`, 7 `WorldData::restrict`, 8 `verify_ammo_flags`, 9 `puncture_ground`, 10
            // `is_valid`, 11 `is_ocean`, 12 `has_preq`, 13 `get_neon`, 14 `add_message`, 15 the shield
            // sound, 16 `Achieve::add_event`, 17 `play_sound`, 18 `Nuke::add_nuke`, 19 `set_seen2`, 20
            // `MessageWin::add_event`, 21 `Nuke::do_damage`, 22 the nuke-hit sound, then the vtable slots
            // `+0x18`, `+0xc`, `+0x8`, `+0xbc`, `+0xb0`, `+0x34`, `+0xb8`, `+0x60` (23 to 30)); `a[31]`,
            // `a[32]` the two values `restrict` writes through its arguments; then the `--seq` runs:
            // `Random::get` (2, `a[33..35]`), `is_ocean` (8, `a[35..43]`), slot `+0xc` (`a[43..51]`),
            // slot `+0x8` (`a[51..59]`), slot `+0xbc` (`a[59..67]`); `a[67]` whether slot `+0xb8` held
            // `ObjectData::is` (1) or its stub (0). Windows: `m` the round (`Ammo`), `c0` the objects, `c1`
            // `div_3_table`, `c2` the world, `c3` the console, `c4` the game, `c5` `int_str_array`, `c6`
            // the message window, `c7` the graphic pieces, `c8` `game_random`; `p4r..p4r+3` leader row `r`'s
            // `+0x8`, `+0x74`, `+0x7b4` and `+0x6928` slices, `p12 + r` the constants leaders' `+0x74`;
            // `p16 p17 p19 p20 p22 p23 p25 p26` the objects of (0,0) (0,1) (1,0) (1,1) (2,0) (2,1) (8,0)
            // (8,1), the units are unread (their type is the object's own, aliased), `p36..p39` the four
            // type records they share, `p40` the world's `+0x134` grid, `p41` its `+0x138` flags,
            // `p42`/`p43` the two records slot `+0xb0` answers.
            //
            // **The port is the sweep's own**: the crate has `Sim::land` (fight.rs) for this function, but
            // over the whole world and not a view, and it walks a different splash (a Chebyshev disc,
            // `combat::splash_cells`, where the original walks the spiral to `radius[k]` *inclusive*, one
            // cell past the disc). What it calls of the crate: `movement::find_angle`,
            // `world::vector_dist`, `world::MOVE_289` (`move_x`/`move_y`, the first 289 entries, ring
            // 8's typo included), `ai_place::circle` (`circle_x`/`circle_y`). The table's rings 9 and
            // 10 (entries 289 to 440) are not in the crate: [`shipped_spiral`] lays them as the
            // executable has them, defects and all.
            struct Env<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                toks: Vec<String>,
                derefs: [Vec<String>; 8],
                n_rand: usize,
                n_ocean: usize,
                n_c: usize,
                n_8: usize,
                n_bc: usize,
                n_b0: usize,
            }

            /// `move_x`/`move_y` as the executable stores them (`.rdata 0x00adcaf0`/`0x00adc400`),
            /// entries 0 to 441: [`crate::world::MOVE_289`], then rings 9 and 10 with the shipped
            /// table's defects. The top row stops one short of the north-east corner; the east
            /// column runs to the south-east corner, which is laid twice; the bottom row stops one
            /// short of the south-west corner; the west column runs from `r - 1` up to `-(r - 1)`;
            /// and each ring closes on a stray cell, `(-9, -10)` and `(-10, 0)`. Entry 441, which
            /// `radius[10] = 441` makes the last the splash walk reads, is a stray word (not a
            /// cell): laid as one far off the map.
            fn shipped_spiral() -> Vec<(i32, i32)> {
                let mut t: Vec<(i32, i32)> = crate::world::MOVE_289.to_vec();
                for r in [9, 10] {
                    for x in -r..r {
                        t.push((x, -r));
                    }
                    for y in -r + 1..=r {
                        t.push((r, y));
                    }
                    t.push((r, r));
                    for x in (-(r - 1)..r).rev() {
                        t.push((x, r));
                    }
                    for y in (-(r - 1)..r).rev() {
                        t.push((-r, y));
                    }
                    t.push(if r == 9 { (-9, -10) } else { (-10, 0) });
                }
                t.push((1 << 24, 0));
                t
            }

            fn n(v: i32) -> String {
                v.to_string()
            }

            fn hex4(v: i32) -> String {
                v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
            }

            fn i16at(w: &[u8], off: usize) -> i32 {
                i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
            }

            /// `(x ^ s) - s`, the original's absolute value (`i32::MIN` stays negative).
            fn abs_sign(v: i32) -> i32 {
                let s = v >> 31;
                (v ^ s).wrapping_sub(s)
            }

            /// The eight objects the draw lays: (player, index, window).
            const OBJS: [(i32, i32, usize); 8] = [
                (0, 0, 16),
                (0, 1, 17),
                (1, 0, 19),
                (1, 1, 20),
                (2, 0, 22),
                (2, 1, 23),
                (8, 0, 25),
                (8, 1, 26),
            ];
            /// Their type records (a unit's is the same record).
            const TYPE_OF: [usize; 8] = [36, 37, 36, 37, 38, 39, 39, 38];

            impl Env<'_> {
                fn f(&self, off: usize) -> i32 {
                    rd(self.m, off)
                }
                fn tok(&mut self, stub: usize, ecx: &str, args: &[String]) {
                    let mut s = format!("s{stub}:{ecx}");
                    for v in args {
                        s.push(',');
                        s.push_str(v);
                    }
                    self.toks.push(s);
                }
                fn obj(&self, pl: i32, ix: i32) -> usize {
                    OBJS.iter()
                        .find(|o| o.0 == pl && o.1 == ix)
                        .map(|o| o.2)
                        .expect("an object the draw lays")
                }
                fn ty(&self, pl: i32, ix: i32) -> usize {
                    let k = OBJS
                        .iter()
                        .position(|o| o.0 == pl && o.1 == ix)
                        .expect("an object the draw lays");
                    TYPE_OF[k]
                }
                fn div3(&self, i: i32) -> i32 {
                    rd(&self.c[1], 4 * i as usize)
                }
                fn semaphore(&self) -> bool {
                    self.c[4][0x821] & 2 != 0
                }
                fn hits_inc(&mut self, who: i32) {
                    let at = 0x7fc + 4 * who as usize;
                    let v = rd(&self.c[4], at);
                    wr(&mut self.c[4], at, v.wrapping_add(1));
                }
                fn close(&mut self) {
                    self.tok(5, "m+0x0", &[]);
                }
                fn domain(&self) -> i32 {
                    let (pl, ix) = (self.f(0x48), self.f(0x4c));
                    if ix >= 0 && pl >= 0 {
                        rd(&self.p[self.ty(pl, ix)], 0x218)
                    } else {
                        0
                    }
                }
                /// `hit_target`, and `check_hit` when it answers 0.
                fn hit_and_check(&mut self, dom: i32) {
                    self.tok(2, "m+0x0", &[]);
                    if self.a[2] == 0 {
                        self.tok(3, "m+0x0", &[n(dom)]);
                        // `check_hit`'s stub lays the target (1, 1).
                        wr(self.m, 0x4c, 1);
                        wr(self.m, 0x48, 1);
                    }
                }
                fn rand(&mut self) -> i32 {
                    self.tok(6, "c8+0x0", &[n(0), n(0xffff)]);
                    let r = self.a[33 + self.n_rand.min(1)];
                    self.n_rand += 1;
                    r
                }
                fn is_ocean(&mut self, x: i32, y: i32) -> bool {
                    self.derefs[4].push(hex4(x));
                    self.derefs[5].push(hex4(y));
                    let r = self.a[35 + self.n_ocean.min(7)];
                    self.n_ocean += 1;
                    r != 0
                }
                /// The owner byte of cell `(x, y)` of the `+0x134` grid.
                fn owner_at(&self, x: i32, y: i32) -> i32 {
                    let w = rd(&self.c[2], 0);
                    let at = (w.wrapping_mul(y).wrapping_add(x)) as usize * 0x1c + 0xf;
                    i32::from(self.p[40][at] as i8)
                }
                fn do_damage_call(&mut self, tgt: (i32, i32), angle: i32, count: i32, flag: i32) {
                    let shooter = self.obj(self.f(0x3c), self.f(0x40));
                    let args = [
                        n(tgt.0),
                        n(tgt.1),
                        n(angle),
                        n(self.f(0x44)),
                        n(self.f(0x34)),
                        n(count),
                        n(flag),
                        n(0),
                    ];
                    self.tok(4, &format!("p{shooter}+0x0"), &args);
                }
                /// One of the two missile-shield messages (`get_neon`, `add_message`, the sound).
                fn shield_message(&mut self, str_off: i32, cx: i32, cy: i32) {
                    let team = i32::from(self.p[4 * self.f(0x3c) as usize + 3][0]);
                    self.tok(13, &n(0xe39ce0 + team * 0x64), &["sp-0x4c".to_string()]);
                    let color = self.a[13];
                    let x = cx.wrapping_mul(3).wrapping_shl(8).wrapping_add(0x180);
                    let y = cy.wrapping_mul(3).wrapping_shl(8).wrapping_add(0x180);
                    self.tok(
                        14,
                        "c6+0x0",
                        &[
                            n(str_off),
                            n(x),
                            n(y),
                            n(1),
                            n(color),
                            n(-1),
                            "c3+0x0".to_string(),
                            n(0),
                            n(0),
                            "c3+0x0".to_string(),
                        ],
                    );
                    self.tok(15, "c6+0x0", &[n(0x9a)]);
                }

                fn run(&mut self) {
                    // Two log lines.
                    let s = rd(&self.c[5], 0x10);
                    let game_log = n(0xeb1360);
                    let (f0c, f10, f14) = (self.f(0xc), self.f(0x10), self.f(0x14));
                    self.tok(
                        0,
                        &game_log,
                        &[
                            n(s.wrapping_add(0xa14)),
                            n(0xeb437c),
                            n(f0c),
                            n(f10),
                            n(f14),
                            n(0),
                            n(0),
                            n(0),
                            n(0),
                            n(3),
                        ],
                    );
                    let (f18, f1c, f20) = (self.f(0x18), self.f(0x1c), self.f(0x20));
                    self.tok(
                        0,
                        &game_log,
                        &[
                            n(s.wrapping_add(0xa28)),
                            n(0xeb437c),
                            n(f18),
                            n(f1c),
                            n(f20),
                            n(0),
                            n(0),
                            n(0),
                            n(0),
                            n(3),
                        ],
                    );
                    // A target that is gone is forgotten.
                    if self.f(0x4c) >= 0 && self.f(0x48) >= 0 {
                        let w = self.obj(self.f(0x48), self.f(0x4c));
                        if self.p[w][8] & 1 == 0 {
                            wr(self.m, 0x48, -1);
                            wr(self.m, 0x4c, -1);
                        }
                    }
                    let gp = self.a[1];
                    let f8 = self.f(8);
                    self.tok(1, "c7+0x0", &[n(f8)]);
                    if self.f(0x40) < 0 || self.f(0x3c) < 0 || gp == 9 {
                        return self.close();
                    }
                    if self.f(0x30) == 0 {
                        self.arm_ground();
                    } else {
                        self.arm_splash();
                    }
                }

                /// `field_0x30 == 0`: the round has no splash.
                fn arm_ground(&mut self) {
                    let dom = self.domain();
                    self.hit_and_check(dom);
                    let (ix, pl) = (self.f(0x4c), self.f(0x48));
                    if ix < 0 || pl < 0 || (ix == self.f(0x40) && pl == self.f(0x3c)) {
                        // Nothing struck: a puncture point near the landing.
                        let r1 = self.rand();
                        let x = self.f(0x18).wrapping_add(r1 % 0x29).wrapping_sub(0x14);
                        let r2 = self.rand();
                        let y = self.f(0x1c).wrapping_add(r2 % 0x29).wrapping_sub(0x14);
                        self.derefs[0].push(hex4(x));
                        self.derefs[1].push(hex4(y));
                        let (x, y) = (self.a[31], self.a[32]);
                        let f8 = self.f(8);
                        self.tok(8, "c7+0x0", &[n(f8), n(1)]);
                        if self.a[8] != 0 {
                            self.tok(9, "m+0x0", &[]);
                            self.derefs[2].push(hex4(x));
                            self.derefs[3].push(hex4(y));
                            if self.a[10] != 0 {
                                let w3 = rd(&self.c[2], 0x18);
                                let at = (self
                                    .div3(y >> 6)
                                    .wrapping_mul(w3)
                                    .wrapping_add(self.div3(x >> 6)))
                                    as usize
                                    * 2;
                                if self.p[41][at] & 0x30 != 0x20 {
                                    self.m[4] = 1;
                                    wr(self.m, 0x24, 0);
                                    return;
                                }
                            }
                        }
                        return self.close();
                    }
                    let w = self.obj(pl, ix);
                    if self.p[w][8] & 1 != 0 {
                        let angle = crate::movement::find_angle(
                            self.f(0x18).wrapping_sub(self.f(0xc)),
                            self.f(0x1c).wrapping_sub(self.f(0x10)),
                        )
                        .0;
                        self.do_damage_call((ix, pl), angle, 0x100, 0);
                        if self.semaphore() && pl != self.f(0x3c) {
                            self.hits_inc(self.f(0x3c));
                        }
                    }
                    self.close();
                }

                /// `field_0x30 != 0`: the round splashes.
                fn arm_splash(&mut self) {
                    let dom = self.domain();
                    let (sp, si) = (self.f(0x3c), self.f(0x40));
                    let shooter = self.obj(sp, si);
                    let sty = self.ty(sp, si);
                    let (f18, f1c) = (self.f(0x18), self.f(0x1c));
                    if rd(&self.p[sty], 0x1e4) & 0x8000000 != 0 {
                        // `Object::is(0x13b)`: the shooter is a nuke.
                        let nuke = if self.a[67] != 0 {
                            self.tok(30, &format!("p{sty}+0x0"), &[n(0x13b), n(0)]);
                            self.a[30]
                        } else {
                            self.tok(29, &format!("p{shooter}+0x0"), &[n(0x13b), n(0)]);
                            self.a[29]
                        };
                        let cx = self.div3(f18 >> 8);
                        let cy = self.div3(f1c >> 8);
                        let owner = if !self.is_ocean(cx, cy) {
                            self.owner_at(cx, cy)
                        } else {
                            let mut owner = -1;
                            let w = rd(&self.c[2], 0);
                            let h = rd(&self.c[2], 4);
                            let ring = crate::world::MOVE_289;
                            for &(dx, dy) in &ring[1..121] {
                                let (xx, yy) = (cx + dx, cy + dy);
                                if xx >= 0 && yy >= 0 && xx < w && yy < h && !self.is_ocean(xx, yy)
                                {
                                    owner = self.owner_at(xx, yy);
                                    if owner >= 0 {
                                        break;
                                    }
                                }
                            }
                            owner
                        };
                        let mut shield = false;
                        if owner >= 0 {
                            let ecx = n(0xe3a390 + owner.wrapping_mul(0x6eec));
                            self.tok(12, &ecx, &[n(0x2b5)]);
                            if self.a[12] != 0 && sp != owner {
                                shield = true;
                                let who = rd(&self.c[3], 0x298);
                                if sp == who {
                                    self.shield_message(0x690, cx, cy);
                                }
                                if owner == who {
                                    self.shield_message(0x6a4, cx, cy);
                                }
                            }
                        }
                        if shield {
                            return self.close();
                        }
                        if nuke != 0 {
                            return self.nuke_blast(sp, si);
                        }
                        self.tok(22, "sp-0x60", &[n(0x50), n(f18), n(f1c)]);
                        let c = &mut self.p[4 * sp as usize + 2];
                        let v = rd(c, 0xc);
                        wr(c, 0xc, v.wrapping_add(1));
                    }
                    self.hit_and_check(dom);
                    let (sv_ix, sv_pl) = (self.f(0x4c), self.f(0x48));
                    self.tok(23, &format!("p{shooter}+0x0"), &[]);
                    let (mut px, mut py) = (f18, f1c);
                    if self.a[23] != 0 && rd(&self.p[sty], 0x2b4) & 0x2000 != 0 {
                        px = self.f(0xc).wrapping_add(f18);
                        px = px.wrapping_sub(px >> 31) >> 1;
                        py = self.f(0x10).wrapping_add(f1c);
                        py = py.wrapping_sub(py >> 31) >> 1;
                    }
                    let angle = crate::movement::find_angle(
                        f18.wrapping_sub(self.f(0xc)),
                        f1c.wrapping_sub(self.f(0x10)),
                    )
                    .0;
                    let area = self.f(0x30);
                    let mut ridx = (area.wrapping_add((area >> 31) & 3) >> 2).wrapping_add(1);
                    if ridx >= 11 {
                        ridx = 10;
                    }
                    let limit = (2 * ridx + 1) * (2 * ridx + 1);
                    let (ccx, ccy) = (self.div3(px >> 8), self.div3(py >> 8));
                    let (w, h) = (rd(&self.c[2], 0), rd(&self.c[2], 4));
                    let spiral = shipped_spiral();
                    let mut hit = false;
                    for i in 0..=limit as usize {
                        let (cxx, cyy) = (ccx + spiral[i].0, ccy + spiral[i].1);
                        if cxx < 0 || cyy < 0 || cxx >= w || cyy >= h {
                            continue;
                        }
                        let rec = (cyy.wrapping_mul(w).wrapping_add(cxx)) as usize * 0x1c;
                        let first = i16at(&self.p[40], rec + 8);
                        let first_pl = i16at(&self.p[40], rec + 0xa);
                        if first < 0 {
                            continue;
                        }
                        let (mut pl, mut ix) = (first_pl, first);
                        loop {
                            wr(self.m, 0x48, pl);
                            wr(self.m, 0x4c, ix);
                            let ow = self.obj(pl, ix);
                            let (next_pl, next_ix) =
                                (i16at(&self.p[ow], 0x2e), i16at(&self.p[ow], 0x2c));
                            self.splash_one(pl, ix, (sv_ix, sv_pl), (px, py), dom, angle, &mut hit);
                            if next_ix < 0 {
                                break;
                            }
                            pl = next_pl;
                            ix = next_ix;
                        }
                    }
                    if !hit {
                        return self.close();
                    }
                    self.tok(28, &format!("p{shooter}+0x0"), &[]);
                    if self.f(8) == self.a[28] {
                        return self.close();
                    }
                    self.hits_inc(sp);
                    self.close();
                }

                /// One object on the chain of a cell.
                #[allow(clippy::too_many_arguments)]
                fn splash_one(
                    &mut self,
                    pl: i32,
                    ix: i32,
                    sv: (i32, i32),
                    (px, py): (i32, i32),
                    dom: i32,
                    angle: i32,
                    hit: &mut bool,
                ) {
                    let (sp, si) = (self.f(0x3c), self.f(0x40));
                    if ix == si && pl == sp {
                        return;
                    }
                    if pl >= 8 {
                        return;
                    }
                    let intended = ix == sv.0 && pl == sv.1;
                    if !intended {
                        let row = 4 * sp as usize;
                        let f8 = rd(&self.p[row], 0);
                        if pl == f8 {
                            return;
                        }
                        if rd(&self.p[row + 1], 4 * pl as usize) != 0
                            && rd(&self.p[12 + pl as usize], 4 * f8 as usize) != 0
                        {
                            return;
                        }
                    }
                    let tgt = self.obj(pl, ix);
                    let tgt_ecx = format!("p{tgt}+0x0");
                    self.tok(24, &tgt_ecx, &[]);
                    let r = self.a[43 + self.n_c.min(7)];
                    self.n_c += 1;
                    let flag = i32::from(!intended);
                    let area = self.f(0x30);
                    let den = area.wrapping_mul(0xc0);
                    let k = 0x63637;
                    if r != 0 {
                        if dom == 2 {
                            return;
                        }
                        // The footprint arm.
                        self.tok(27, &tgt_ecx, &[]);
                        let wa = 42 + self.n_b0.min(1);
                        self.n_b0 += 1;
                        let ta = if wa == 42 { 36 } else { 37 };
                        let dx1 = abs_sign(px.wrapping_sub(rd(&self.p[tgt], 0x10) ^ k))
                            .wrapping_sub(rd(&self.p[ta], 0x234).wrapping_mul(0xc0));
                        self.tok(27, &tgt_ecx, &[]);
                        let wb = 42 + self.n_b0.min(1);
                        self.n_b0 += 1;
                        let tb = if wb == 42 { 36 } else { 37 };
                        let dy1 = abs_sign(py.wrapping_sub(rd(&self.p[tgt], 0x14) ^ k))
                            .wrapping_sub(rd(&self.p[tb], 0x238).wrapping_mul(0xc0));
                        let (ex, ey) = (dx1.max(0), dy1.max(0));
                        let dist = crate::world::vector_dist(ex, ey);
                        let cnt = 0x100 - dist.wrapping_shl(8).wrapping_div(den);
                        if cnt < 0 {
                            return;
                        }
                        self.do_damage_call((ix, pl), angle, cnt, flag);
                        if self.semaphore() && pl != sp {
                            *hit = true;
                        }
                        return;
                    }
                    self.tok(25, &tgt_ecx, &[]);
                    let r2 = self.a[51 + self.n_8.min(7)];
                    self.n_8 += 1;
                    if r2 == 0 {
                        return;
                    }
                    self.tok(26, &tgt_ecx, &[]);
                    let r3 = self.a[59 + self.n_bc.min(7)];
                    self.n_bc += 1;
                    if r3 == 0 {
                        return;
                    }
                    let tty = self.ty(pl, ix);
                    if (rd(&self.p[tty], 0x218) != 2) != (dom != 2) {
                        return;
                    }
                    if rd(&self.p[tty], 0x1e4) & 0x8000000 != 0 {
                        return;
                    }
                    let ey = abs_sign(py.wrapping_sub(rd(&self.p[tgt], 0x14) ^ k));
                    let ex = abs_sign(px.wrapping_sub(rd(&self.p[tgt], 0x10) ^ k));
                    let dist = crate::world::vector_dist(ex, ey);
                    let v = (-0xc0_i32)
                        .wrapping_sub(rd(&self.p[tty], 0x23c))
                        .wrapping_add(dist);
                    let v = v.max(0);
                    let cnt = 0x100 - v.wrapping_shl(8).wrapping_div(den);
                    if cnt <= 0 {
                        return;
                    }
                    self.do_damage_call((ix, pl), angle, cnt, flag);
                    if self.semaphore() && pl != sp {
                        *hit = true;
                    }
                }

                /// The nuke's blast: the leader stamped, the effect laid, the ground lit.
                fn nuke_blast(&mut self, sp: i32, si: i32) {
                    let row = 4 * sp as usize + 2;
                    let frame = rd(&self.c[4], 0x550);
                    wr(&mut self.p[row], 0, frame);
                    let v = rd(&self.p[row], 8);
                    wr(&mut self.p[row], 8, v.wrapping_add(1));
                    let nn = rd(&self.p[self.ty(sp, si)], 0x200);
                    let (f18, f1c) = (self.f(0x18), self.f(0x1c));
                    self.tok(
                        18,
                        "sp-0x70",
                        &[
                            n(f18),
                            n(f1c),
                            n(0x64),
                            n(nn.wrapping_mul(3).wrapping_shl(6)),
                            n(sp),
                            n(si),
                        ],
                    );
                    let ring = (nn.wrapping_add(1)).clamp(0, 0x40) as usize;
                    let tx = self.div3(f18 >> 6);
                    let ty = self.div3(f1c >> 6);
                    let circle = crate::ai_place::circle();
                    let (w3, h3) = (rd(&self.c[2], 0x18), rd(&self.c[2], 0x1c));
                    for i in 0..circle.radius[ring] {
                        let y = circle.y[i] + ty;
                        let x = circle.x[i] + tx;
                        if x >= 0 && y >= 0 && x < w3 && y < h3 {
                            self.derefs[6].push(hex4(x >> 1));
                            self.derefs[7].push(hex4(y >> 1));
                            self.tok(
                                19,
                                &n(x >> 1),
                                &["sp-0x2c".to_string(), "sp-0x30".to_string(), n(0xff), n(0)],
                            );
                        }
                    }
                    self.tok(
                        20,
                        "c6+0x0",
                        &[
                            n(f18),
                            n(f1c),
                            n(-1),
                            n(0xc8d26c),
                            n(-1),
                            n(0xc00),
                            n(0),
                            n(0),
                            n(0),
                        ],
                    );
                    self.tok(21, "c6+0x0", &[]);
                    self.close();
                }
            }

            let mut env = Env {
                a,
                m,
                c,
                p,
                toks: Vec::new(),
                derefs: Default::default(),
                n_rand: 0,
                n_ocean: 0,
                n_c: 0,
                n_8: 0,
                n_bc: 0,
                n_b0: 0,
            };
            env.run();
            log.extend(env.toks);
            for d in &env.derefs {
                log.push(if d.is_empty() {
                    "-".to_string()
                } else {
                    d.join("/")
                });
            }
            0
        };
        assert_eq!(got, want_eax, "do_damage — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "do_damage — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_damage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[10 + i], "do_damage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Ammo::do_damage@00678060: {n} rows agree");
}
