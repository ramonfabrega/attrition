//! `Armies::find_local_army@006f32e0` under the emulator against the port (item 3234; tools/sweep_gen.py, obj with windows).

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

/// `Armies::find_local_army@006f32e0`, run by `tools/emu/sweep_armies_find_local_army.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), `p` windows in order: p0..p26 the place(s), in flag order; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00c061e8 points at 2304 random bytes; 4 random bytes at 0x00cb4bac; 96 random bytes at 0x00e3a390; callee 0x006b52e0 (2 args) stubbed; callee 0x0046cff0 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; the word at `0xc09710` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `p0+24` points at 64 random bytes less `0` (placed); the word at `p0+28` points at 64 random bytes less `0` (placed); the word at `p0+32` points at 64 random bytes less `0` (placed); the word at `p0+36` points at 64 random bytes less `0` (placed); the word at `p0+40` points at 64 random bytes less `0` (placed); the word at `p0+44` points at 64 random bytes less `0` (placed); the word at `p0+48` points at 64 random bytes less `0` (placed); the word at `p0+52` points at 64 random bytes less `0` (placed); the word at `p0+56` points at 64 random bytes less `0` (placed); the word at `p0+60` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 32 random bytes less `0` (placed); the word at `p17+4` points at 32 random bytes less `0` (placed); the word at `p17+8` points at 32 random bytes less `0` (placed); the word at `p17+12` points at 32 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `p18+0x18` points at 704 random bytes less `0` (placed); the word at `p19+0x18` points at 704 random bytes less `0` (placed); the word at `p20+0x18` points at 704 random bytes less `0` (placed); the word at `p21+0x18` points at 704 random bytes less `0` (placed); slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 0 answered from [0, 1, 2] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 1 answers 0..150 every row; stub 2 answers 0..4 every row; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_find_local_army.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // c0 the world cell, c1 `gamec`, c2 `find_dist`, c3 the leaders (player 0); p1..p16 the armies,
                // p23..p26 the units' data; a[5] the region `get_tregion` answers, a[6] `vector_dist`, a[7]
                // `Army::count`, a[8] / a[9] the slot +0xcc / +0xc4 answers, a[10] / a[11] the flags laying
                // `is_supply` / `is_hero`.
                let tre = a[5];
                let who = a[0];
                let mut best = -1i32;
                let mut near = 99_999_999i32;
                c[2][..4].copy_from_slice(&near.to_le_bytes());
                let (mut l1, mut l2) = (Vec::new(), Vec::new());
                let lead = rd(&c[3], 0x50);
                let g = &c[1];
                let diff = if g[0x820] & 4 != 0 {
                    lead
                } else if g[0x821] & 0x10 != 0 && g[0x822] & 2 == 0 {
                    if lead < 0 { i32::from(g[0x2b]) } else { lead }
                } else {
                    i32::from(g[0x2b])
                };
                for k in 0..16usize {
                    let r = &p[1 + k];
                    if i16::from_le_bytes([r[0], r[1]]) == 0 || rd(r, 8) != tre {
                        continue;
                    }
                    let tgt = rd(r, 0x34);
                    if tgt >= 0 && tgt != who {
                        continue;
                    }
                    if !(diff > 2 || r[4] & 1 != 0) {
                        continue;
                    }
                    let dx = a[1].wrapping_sub(rd(r, 0x38)).wrapping_abs();
                    let dy = a[2].wrapping_sub(rd(r, 0x3c)).wrapping_abs();
                    l1.push(format!("{dx},{dy}"));
                    let dist = if rd(r, 0x10) == 0 { 90_000_000 } else { a[6] };
                    if dist > near {
                        continue;
                    }
                    let mut take = a[4] < 0;
                    if !take {
                        let u = a[4] as usize;
                        let flags = rd(&p[23 + u], 0x2b8);
                        let supply = if a[10] != 0 {
                            flags & 0x40 != 0
                        } else {
                            a[8] != 0
                        };
                        if supply {
                            l2.push(format!("p{}+0x0,{},17,63", 1 + k, a[4]));
                            take = a[7] <= 2;
                        } else {
                            let hero = if a[11] != 0 {
                                flags & 0x20 != 0
                            } else {
                                a[9] != 0
                            };
                            if hero {
                                l2.push(format!("p{}+0x0,{},17,54", 1 + k, a[4]));
                                take = a[7] <= 1;
                            } else {
                                take = true;
                            }
                        }
                    }
                    if take {
                        best = k as i32;
                        near = dist;
                        c[2][..4].copy_from_slice(&near.to_le_bytes());
                    }
                }
                let j = |l: Vec<String>| {
                    if l.is_empty() {
                        "-".to_string()
                    } else {
                        l.join("/")
                    }
                };
                log.push(j(l1));
                log.push(j(l2));
                i64::from(best)
            }
        };
        assert_eq!(got, want_eax, "find_local_army — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_local_army — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_local_army — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_local_army — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "find_local_army — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Armies::find_local_army@006f32e0: {n} rows agree");
}
