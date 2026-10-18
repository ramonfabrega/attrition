//! `Army::engagement@006f5160` under the emulator against the port (item 3239; tools/sweep_gen.py, obj with windows).

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

/// `Army::engagement@006f5160`, run by `tools/emu/sweep_army_engagement.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 sparse table 0 (2 rows × 2 slice(s), row-major), p4..p7 sparse table 1 (4 rows × 1 slice(s), row-major), p8..p27 the place(s), in flag order, p28..p37 the stubs' answer window(s); the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00e85f20 points at 12288 random bytes; the global cell 0x00e41248 points at 256 random bytes; the global cell 0x00e48134 points at 256 random bytes; callee 0x006f9df0 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x006f8a20 (2 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; callee 0x0060a870 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; callee 0x006f8750 (1 args) stubbed; callee 0x0070d7e0 (0 args) stubbed; callee 0x00712490 (5 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 32 random bytes less `0` (placed); the word at `p8+4` points at 32 random bytes less `0` (placed); the word at `p8+8` points at 32 random bytes less `0` (placed); the word at `p8+12` points at 32 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 32 random bytes less `0` (placed); the word at `p13+4` points at 32 random bytes less `0` (placed); the word at `p13+8` points at 32 random bytes less `0` (placed); the word at `p13+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p18+0` points at 32 random bytes less `0` (placed); the word at `p18+4` points at 32 random bytes less `0` (placed); the word at `p18+8` points at 32 random bytes less `0` (placed); the word at `p18+12` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p23+0` points at 32 random bytes less `0` (placed); the word at `p23+4` points at 32 random bytes less `0` (placed); the word at `p23+8` points at 32 random bytes less `0` (placed); the word at `p23+12` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 4 answered from [10, 10, 10, 10, 10, 10, 9] (90%); stub 9 answered from [1, 0, 0, 0, 0, 0, 0] (90%); stub 10 answered from [0, 0, 1] (90%); stub 12 answered from [1, 1, 0] (90%); stub 3 answers the address of 32 random bytes, a `p` window after the places; stub 7 answers the address of 32 random bytes, a `p` window after the places; stub 8 answers a window of its own to each call, in call order (16 bytes, ...); stub 0 answers 4 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 9 answers 4 values in call order, printed after the answers; stub 12 answers 5 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 writes a value in [0, 1, 2, 3] through its argument 0, printed after the answers (`o` in a place); stub 5 writes a value in [0, 1, 2, 3] through its argument 1, printed after the answers (`o` in a place); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); 4 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x20 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_engagement.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0..=13 the single answers of stubs 0..=13 (only 10, `count`, is read), 14 and 15 what
                // `center_of_gravity` writes through its arguments 0 (pairs with the object's +0x10) and 1 (+0x14),
                // then the runs in call order: 16.. `get_unit` (4), 20.. slot +0x8, 24.. slot +0xbc, 28.. the order's
                // slot +0x10, 32.. `is_map_unit`, 36.. `num_valid` (5). Windows: p0..p3 the leaders table's slices
                // (player 0: +0x8 / +0x74.., player 1), p4..p7 the leaders-const rows (+0x74..), p19..p22 and p24..p27
                // the `objects` records of type 0 / 1, p30..p37 what slot +0x3c answers a call (ox at +8, whom at
                // +0xc), c1 the groups, c2 / c3 the leader record of player 0 / 1 (+0xe8). Calls are logged for
                // stubs 9..=13 in stub order. `vector_dist` is the original's own code (stub 6).
                const KEY: i32 = 0x63637;
                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                let count = rd(m, 0x10);
                let (mut l9, mut l10, mut l11, mut l12, mut l13): (Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>) =
                    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                let player = sh(m, 0x94);
                if count > 0 {
                    let (mut ox, mut whom) = (-1i32, -1i32);
                    let (mut k0, mut k1, mut k2, mut k4, mut k9, mut slot3c) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
                    let mut cog = false;
                    for _ in 0..count {
                        let u = a[16 + k0.min(3)] as usize;
                        k0 += 1;
                        let s8 = a[20 + k1.min(3)];
                        k1 += 1;
                        if s8 == 0 {
                            continue;
                        }
                        let sbc = a[24 + k2.min(3)];
                        k2 += 1;
                        if sbc == 0 {
                            continue;
                        }
                        let ord = a[28 + k4.min(3)];
                        k4 += 1;
                        if ord != 10 {
                            continue;
                        }
                        let r = &p[(if player & 1 == 0 { 19 } else { 24 }) + u];
                        let (ry, rx) = (rd(r, 0x10) ^ KEY, rd(r, 0x14) ^ KEY);
                        let d = crate::world::vector_dist(
                            ry.wrapping_sub(rd(m, 0x38)).wrapping_abs(),
                            rx.wrapping_sub(rd(m, 0x3c)).wrapping_abs(),
                        );
                        if d >= 0xc00 {
                            continue;
                        }
                        cog = true;
                        let d2 = crate::world::vector_dist(
                            a[14].wrapping_mul(0x300).wrapping_sub(ry).wrapping_add(0x180).wrapping_abs(),
                            a[15].wrapping_mul(0x300).wrapping_sub(rx).wrapping_add(0x180).wrapping_abs(),
                        );
                        if d2 > 0xc00 {
                            continue;
                        }
                        ox = rd(&p[30 + slot3c.min(7)], 8);
                        whom = rd(&p[30 + (slot3c + 1).min(7)], 0xc);
                        slot3c += 2;
                        if ox >= 0 && whom >= 0 {
                            let w = (if whom == 0 { 19 } else { 24 }) + ox as usize;
                            l9.push(format!("p{w}+0x0"));
                            let ans = a[32 + k9.min(3)];
                            k9 += 1;
                            if ans != 0 {
                                break;
                            }
                        }
                    }
                    let _ = cog;
                    if ox >= 0 && whom >= 0 {
                        let tl = rd(m, 0x34);
                        let pl = (player & 1) as usize;
                        if tl >= 0 {
                            let l8 = rd(&p[2 * pl], 0);
                            if tl != l8 {
                                let ta = usize::try_from(tl).expect("leader");
                                let a_zero = rd(&p[2 * pl + 1], 4 * ta) == 0;
                                let b_zero = rd(&p[4 + ta], 4 * usize::try_from(l8).expect("eight")) == 0;
                                if (a_zero || b_zero) && rd(m, 0x2c) == 0 && (rd(&c[2 + pl], 0xe8) ^ 0x63187) <= 3 {
                                    l10.push("m+0x0,4,0".to_string());
                                    if a[10] == 0 {
                                        l11.push("m+0x0,3".to_string());
                                    }
                                }
                            }
                        }
                        let ng = sh(m, 0x96);
                        let mut k12 = 0usize;
                        for i in 0..ng.max(0) as usize {
                            let g = rd(m, 0x54 + 4 * i);
                            if g < 0 {
                                continue;
                            }
                            l12.push(format!("c1+{:#x}", g * 0x9d4));
                            let nv = a[36 + k12.min(4)];
                            k12 += 1;
                            if nv != 0 {
                                l13.push(format!("c1+{:#x},{ox},{whom},0,2,0", g * 0x9d4));
                            }
                        }
                    }
                }
                for l in [l9, l10, l11, l12, l13] {
                    log.push(if l.is_empty() { "-".to_string() } else { l.join("/") });
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "engagement — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "engagement — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "engagement — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "engagement — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Army::engagement@006f5160: {n} rows agree");
}
