//! `Unit::think@005f6e40` under the emulator against the port (item 3346; tools/sweep_gen.py, obj with windows).

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

/// `Unit::think@005f6e40`, run by `tools/emu/sweep_unit_think.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p16 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c06210 points at 680 random bytes; the global cell 0x00c06208 points at 16 random bytes; the global cell 0x00c061c4 points at 8 random bytes; the global cell 0x00e85f20 points at 7424 random bytes; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xf4 (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; callee 0x005e5410 (5 args) stubbed; callee 0x00648d70 (4 args) stubbed; callee 0x005f4740 (0 args) stubbed; callee 0x005f5a80 (0 args) stubbed; callee 0x0060a410 (0 args) stubbed; callee 0x0046fa10 (0 args) stubbed; callee 0x005f5760 (1 args) stubbed; callee 0x005f5650 (1 args) stubbed; callee 0x0046fae0 (0 args) stubbed; callee 0x0046d370 (0 args) stubbed; callee 0x005fce20 (7 args) stubbed; callee 0x006038e0 (1 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x005f27a0 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x005f6010 (1 args) stubbed; callee 0x005f4c60 (0 args) stubbed; callee 0x005f2ee0 (0 args) stubbed; callee 0x00649d70 (0 args) stubbed; callee 0x005f7740 (0 args) stubbed; the word at `m+0x18` points at 736 random bytes less `0` (placed); the word at `c1+0x14` points at 16 random bytes less `0` (placed); the word at `c1+0x30` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p9+4` points at 16 random bytes less `0` (placed); the word at `p9+8` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p10+8` points at 16 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 1 answered from [0, 0, 1] (90%); stub 2 answered from [0, 0, 1, 1] (90%); stub 5 answered from [0, 0, 1] (90%); stub 6 answered from [0, 0, 1] (90%); stub 7 answered from [0, 1, 1] (90%); stub 8 answered from [0, 0, 1] (90%); stub 9 answered from [0, 0, 1] (90%); stub 11 answered from [0, 1, 1, 1] (90%); stub 12 answered from [0, 0, 0, 1] (90%); stub 13 answered from [0, 0, 0, 0, 1] (90%); stub 14 answered from [0, 0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 16 answered from [0, 0, 0, 1] (90%); stub 17 answered from [0, 0, 0, 1] (90%); stub 18 answered from [0, 1, 1] (90%); stub 19 answered from [0, 1, 1] (90%); stub 20 answered from [0, 1, 1] (90%); stub 21 answered from [0, 0, 1] (90%); stub 25 answered from [0, 0, 0, 1] (90%); stub 26 answered from [0, 0, 1] (90%); stub 27 answered from [0, 1] (90%); stub 28 answered from [0, 0, 1] (90%); stub 29 answered from [0, 0, 1] (90%); stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 12 answers 2 values in call order, printed after the answers; stub 13 answers 3 values in call order, printed after the answers; stub 19 answers 2 values in call order, printed after the answers; stub 27 answers 2 values in call order, printed after the answers; stub 0 answers 1..1 every row; stub 3 answers -3..9 every row; stub 4 answers 0..3 every row; stub 30 answers -2..2 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x8, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 12000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **`Unit::think@005f6e40`, the captain's dispatcher, as the original orders it.** No crate method can
                // be called over windows: `Sim::think` (crates/sim/src/orders.rs) reads `self.units` and the
                // `Sim::think_*` it calls are the real ones, so this is the inline port, written from the listing in
                // the original's order of tests and calls. The non-captain half (the captain mirror) is a sibling.
                //
                // `a`: no stack argument; then one answer a stub (`a[k]` for stub k: 0 `is_captain` slot +0xe8,
                // 1 the unit's `is` +0xb8, 2 the type's `is` +0x60, 3 stance +0xf4, 4 packer stance +0x100, 5 `is_caravan`
                // +0xd0, 6 the type's +0x130, 7 `is_special` +0xd4, 8 `is_supply` +0xcc, 9 `is_hero` +0xc4, 10
                // `add_attack_order`, 11 `is_in_range`, 12 `think_merchant`, 13 `think_attack`, 14
                // `is_packing_or_unpacking`, 15 `is_worker`, 16 `think_peasant`, 17 `think_caravan`, 18
                // `is_rare_collector`, 19 `is_merchant`, 20 `do_gather`, 21 `unpack_merchant`, 22 `get_neon`, 23
                // `add_message`, 24 `play`, 25 `think_spellcaster`, 26 `GroupData::member`, 27 `think_scout`, 28
                // `think_fish`, 29 `think_carry`, 30 `get_army`, 31 `add_to_army`); then the `--seq` stubs' answers a
                // call (stub 1 four, 2 four, 12 two, 13 three, 19 two, 27 two); then the six `--vtat-keep` flags
                // (is_captain, `is`, `is_caravan`, `is_special`, `is_supply`, `is_hero`: 1 = the image's own function
                // sits in the slot, so the original inlines it).
                struct Ctx<'l> {
                    a: &'l [i32],
                    pos: [usize; 32],
                    log: &'l mut Vec<String>,
                }
                impl Ctx<'_> {
                    /// One stubbed call: the log token and the answer (a `--seq` stub's next, else its one).
                    fn call(&mut self, k: usize, ecx: &str, args: &[i32]) -> i32 {
                        self.call_s(k, ecx, args, "")
                    }
                    /// The same with a stack token (`sp-0x28`, a caller's local) as the first argument.
                    fn call_s(&mut self, k: usize, ecx: &str, args: &[i32], lead: &str) -> i32 {
                        let mut t = format!("s{k}:{ecx}");
                        if !lead.is_empty() {
                            t.push_str(&format!(",{lead}"));
                        }
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                        self.ans(k)
                    }
                    fn ans(&mut self, k: usize) -> i32 {
                        let (base, n) = match k {
                            1 => (32, 4),
                            2 => (36, 4),
                            12 => (40, 2),
                            13 => (42, 3),
                            19 => (45, 2),
                            27 => (47, 2),
                            _ => return self.a[k],
                        };
                        let i = self.pos[k].min(n - 1);
                        self.pos[k] += 1;
                        self.a[base + i]
                    }
                    fn based(&self, j: usize) -> bool {
                        self.a[49 + j] != 0
                    }
                    /// `is(ty, flag)`: the unit's slot +0xb8 (a stub) or, when the slot holds `ObjectData::is`, the
                    /// type's slot +0x60 (a stub) called with the same two.
                    fn is(&mut self, ty: i32, fl: i32) -> i32 {
                        if self.based(1) {
                            self.call(2, "p8+0x0", &[ty, fl])
                        } else {
                            self.call(1, "m+0x0", &[ty, fl])
                        }
                    }
                    /// The console message: `get_neon`, `add_message` (string offset, position, colour, the player
                    /// twice around two zeros) and the sound.
                    fn announce(&mut self, neon_ecx: i32, xy: (i32, i32), str_off: i32, lead: i32, snd: i32) {
                        let neon = self.call_s(22, &neon_ecx.to_string(), &[], "sp-0x1c");
                        self.log.push(format!(
                            "s23:c3+0x0,{str_off},{},{},1,{neon},1,{lead},0,0,{lead}",
                            xy.0, xy.1
                        ));
                        self.log.push(format!("s24:c3+0x0,{snd}"));
                    }
                }
                fn think(cx: &mut Ctx, m: &mut [u8], c: &[Vec<u8>], p: &mut [Vec<u8>]) {
                    let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                    let who = usize::from(m[9]);
                    let o = sh(m, 0xa);
                    let phase = o.wrapping_add(rd(&c[0], 0x550));
                    // The captain test; the non-captain half is the mirror's module.
                    let captain = if cx.based(0) {
                        u16::from_le_bytes([m[0x8e], m[0x8f]]) >> 15 != 0
                    } else {
                        cx.a[0] != 0
                    };
                    assert!(captain, "the non-captain half is the mirror's module");
                    // The exit that clears the "recently ordered" bit.
                    macro_rules! exit_clear {
                        () => {{
                            m[8] &= 0xef;
                            return;
                        }};
                    }
                    // `5f6f7d`: a citizen drops its carry nibble and the leader takes 0x80000.
                    let ty4 = rd(&p[8], 4);
                    if ty4 == 0x32 || ty4 == 0x33 {
                        let f = rd(m, 0x68) & 0x87ff_ffffu32 as i32;
                        wr(m, 0x68, f);
                        let w0 = rd(&p[2 * who], 0) | 0x80000;
                        wr(&mut p[2 * who], 0, w0);
                    }
                    // The cadence gate (one frame in sixteen past an idle of two), the leader's bit, the mask bit.
                    if m[8] & 0x10 == 0 && m[0xb0] > 2 && phase & 0xf != 0 {
                        return;
                    }
                    if rd(&p[2 * who], 4) & 2 != 0 || rd(m, 0x68) & 0x100_0000 != 0 {
                        return;
                    }
                    'seven: {
                        if rd(m, 0x68) & 0x100 != 0 {
                            break 'seven;
                        }
                        if m[0xb0] != 1 && phase & 0x1f != 0 {
                            // `5f7017`: the off-phase `near_o` arm.
                            let (s34, s36) = (sh(m, 0x34), sh(m, 0x36));
                            if captain && s34 >= 0 && s36 >= 0 && cx.call(3, "m+0x0", &[]) < 5 {
                                let alive = p[11 + 3 * s36 as usize + s34 as usize][8] & 1 != 0;
                                if alive && cx.call(11, "m+0x0", &[s34, s36, s36 * 7, 0]) != 0 {
                                    if rd(&p[8], 0x2b8) & 4 == 0
                                        || rd(m, 0x68) & 0x80000 == 0
                                        || cx.call(4, "m+0x0", &[]) == 0
                                    {
                                        cx.call(10, "m+0x0", &[s34, s36, 2, 0, 0]);
                                    }
                                    exit_clear!();
                                }
                            }
                            break 'seven;
                        }
                        // `5f70fd`: the auto-attack arm.
                        let mut hit = false;
                        if cx.is(0x3e, 1) != 0 {
                            if rd(m, 0x68) & 0x80000 != 0 && cx.call(12, "m+0x0", &[]) != 0 {
                                hit = true;
                            } else if cx.call(14, "m+0x0", &[]) == 0 && cx.call(13, "m+0x0", &[]) != 0 {
                                hit = true;
                            }
                        }
                        if !hit
                            && rd(&p[8], 0x1e8) != 0
                            && rd(&p[8], 0x2c8) & 0x10000 != 0
                            && cx.call(13, "m+0x0", &[]) != 0
                        {
                            hit = true;
                        }
                        if hit {
                            exit_clear!();
                        }
                    }
                    // `5f7179`: a leader without bit 2 ends the think here, the bit left as it is.
                    if rd(&p[2 * who], 0) & 2 == 0 {
                        return;
                    }
                    if cx.call(15, "m+0x0", &[]) != 0 && cx.call(16, "m+0x0", &[0]) != 0 {
                        exit_clear!();
                    }
                    let caravan = if cx.based(2) {
                        cx.call(6, "p8+0x0", &[])
                    } else {
                        cx.call(5, "m+0x0", &[])
                    };
                    if caravan != 0 && cx.call(17, "m+0x0", &[0]) != 0 {
                        exit_clear!();
                    }
                    let cwho = rd(&c[2], 0x298);
                    let xy = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
                    let neon_ecx = 0xe3_9ce0 + 100 * i32::from(p[2 * who + 1][0]);
                    // The human rare collector's deploy.
                    if cx.call(18, "m+0x0", &[]) != 0
                        && rd(&p[2 * who], 0) & 4 != 0
                        && rd(m, 0x68) & 0x80000 != 0
                        && (m[0xb0] == 1 || phase & 0x1f == 0)
                        && (rd(m, 0x68) & 0x100 == 0 || cx.call(19, "m+0x0", &[]) != 0)
                    {
                        // `do_gather(&local, 0, 0, 1, 1, -1, -1)` and its tail.
                        let g = cx.call_s(20, "m+0x0", &[0, 0, 1, 1, -1, -1], "sp-0x28");
                        if g != 0 {
                            if cx.call(21, "m+0x0", &[4]) != 0 {
                                exit_clear!();
                            }
                            if i32::from(m[9]) == cwho && m[0xb0] == 1 {
                                cx.announce(neon_ecx, xy, 0x11f30, i32::from(m[9]), 64);
                            }
                        }
                        m[0xb0] = m[0xb0].wrapping_add(1);
                    }
                    // `5f730d`: the computer block.
                    let computer = rd(&p[2 * who], 0) & 4;
                    if computer != 0 || rd(&c[4], 0) != 0 {
                        if computer != 0 {
                            if rd(m, 0x68) & 0x100 == 0 {
                                let special = if cx.based(3) {
                                    rd(&p[8], 0x2b8) & 0x10
                                } else {
                                    cx.call(7, "m+0x0", &[])
                                };
                                if special != 0 {
                                    if (m[0xb0] == 1 || phase & 0x1f == 0) && cx.call(25, "m+0x0", &[]) != 0 {
                                        exit_clear!();
                                    }
                                    let s80 = sh(m, 0x80);
                                    let grp_ok = s80 < 0 || rd(&c[5], s80 as usize * 0x9d4 + 0xc) == 1;
                                    if m[0xb0] == 1 && grp_ok && rd(m, 0x68) & 0x400_0000 != 0 {
                                        let f = rd(m, 0x68) & 0xfbff_ffffu32 as i32;
                                        wr(m, 0x68, f);
                                        if i32::from(m[9]) == cwho {
                                            let play = rd(&c[2], 0x2a0).wrapping_mul(0x9f0);
                                            let mem = cx.call(26, &play.to_string(), &[o, i32::from(m[9]), 1]);
                                            if mem == 0 {
                                                cx.announce(neon_ecx, xy, 0x11f44, play, 101);
                                            }
                                        }
                                    }
                                }
                            } else {
                                cx.call(27, "m+0x0", &[0]);
                            }
                        }
                        if rd(m, 0x68) & 0x40000 == 0 {
                            exit_clear!();
                        }
                    }
                    // `5f748e`: the tail, on its own cadence.
                    if m[0xb0] != 1 && phase & 0x1f != 0 {
                        exit_clear!();
                    }
                    if cx.is(0x13d, 0) != 0 && cx.call(28, "m+0x0", &[]) != 0 {
                        exit_clear!();
                    }
                    if (m[0xb0] == 1 || phase & 0x7f == 0) && cx.call(19, "m+0x0", &[]) != 0 && cx.call(12, "m+0x0", &[]) != 0
                    {
                        exit_clear!();
                    }
                    if rd(&p[2 * who], 0) & 8 != 0 {
                        exit_clear!();
                    }
                    if rd(&p[8], 0x2d4) != 0 && cx.is(0x15f, 1) == 0 && cx.call(29, "m+0x0", &[]) != 0 {
                        exit_clear!();
                    }
                    let supply = if cx.based(4) {
                        rd(&p[8], 0x2b8) & 0x40
                    } else {
                        cx.call(8, "m+0x0", &[])
                    };
                    let hero = if supply != 0 {
                        0
                    } else if cx.based(5) {
                        rd(&p[8], 0x2b8) & 0x20
                    } else {
                        cx.call(9, "m+0x0", &[])
                    };
                    if supply != 0 || hero != 0 {
                        cx.call(31, "m+0x0", &[]);
                        exit_clear!();
                    }
                    if rd(&p[8], 0x2c8) & 0x10 == 0 && cx.is(0x3a, 0) == 0 {
                        exit_clear!();
                    }
                    if cx.call(30, "m+0x0", &[]) < 0 {
                        cx.call(27, "m+0x0", &[0]);
                    }
                    exit_clear!();
                }
                let mut cx = Ctx {
                    a,
                    pos: [0; 32],
                    log: &mut log,
                };
                think(&mut cx, m, c, p);
                if cx.log.is_empty() {
                    cx.log.push("-".to_string());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "think — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "think — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 12000, "only {n} rows");
    eprintln!("Unit::think@005f6e40: {n} rows agree");
}
