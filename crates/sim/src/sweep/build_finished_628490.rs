//! `Build::finished@00628490` under the emulator against the port (item 3282; tools/sweep_gen.py, obj with windows).

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

/// `Build::finished@00628490`, run by `tools/emu/sweep_build_finished_628490.py` under unicorn
/// (tools/emu): 128 random bytes of object, 1 argument(s), `p` windows in order: p0..p20 sparse table 0 (3 rows × 7 slice(s), row-major), p21..p68 the place(s), in flag order; the global cell 0x00c061fc points at 32 random bytes; the global cell 0x00c061f4 points at 32 random bytes; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00c061ec points at 1792 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 528 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x84 (2 args) stubbed; vtable slot +0x80 (2 args) stubbed; callee 0x006dcb60 (5 args) stubbed; callee 0x006e07d0 (2 args) stubbed; callee 0x006dca50 (1 args) stubbed; callee 0x00645330 (1 args) stubbed; callee 0x006454a0 (0 args) stubbed; callee 0x0062f9b0 (1 args) stubbed; callee 0x006e0bc0 (1 args) stubbed; callee 0x00676ce0 (4 args) stubbed; callee 0x00738450 (0 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x007384c0 (0 args) stubbed; callee 0x006dc490 (0 args) stubbed; callee 0x00680f60 (0 args) stubbed; callee 0x006e0600 (1 args) stubbed; callee 0x0065ca80 (13 args) stubbed; the word at `0xe85ddc` points at 24 random bytes less `0` (placed); the word at `p21+0` points at 16 random bytes less `0` (placed); the word at `p21+4` points at 16 random bytes less `0` (placed); the word at `p21+8` points at 16 random bytes less `0` (placed); the word at `p21+12` points at 16 random bytes less `0` (placed); the word at `p21+16` points at 16 random bytes less `0` (placed); the word at `p21+20` points at 16 random bytes less `0` (placed); the word at `0xc0a274` points at 24 random bytes less `0` (placed); the word at `p28+0` points at 756 random bytes less `0` (placed); the word at `p28+4` points at 756 random bytes less `0` (placed); the word at `p28+8` points at 756 random bytes less `0` (placed); the word at `p28+12` points at 756 random bytes less `0` (placed); the word at `p28+16` points at 756 random bytes less `0` (placed); the word at `p28+20` points at 756 random bytes less `0` (placed); the word at `c0+0x10` points at 24 random bytes less `0` (placed); the word at `p35+0` points at 704 random bytes less `0` (placed); the word at `p35+4` points at 704 random bytes less `0` (placed); the word at `p35+8` points at 704 random bytes less `0` (placed); the word at `p35+12` points at 704 random bytes less `0` (placed); the word at `p35+16` points at 704 random bytes less `0` (placed); the word at `p35+20` points at 704 random bytes less `0` (placed); the word at `c1+0x10` points at 24 random bytes less `0` (placed); the word at `p42+0` points at 708 random bytes less `0` (placed); the word at `p42+4` points at 708 random bytes less `0` (placed); the word at `p42+8` points at 708 random bytes less `0` (placed); the word at `p42+12` points at 708 random bytes less `0` (placed); the word at `p42+16` points at 708 random bytes less `0` (placed); the word at `p42+20` points at 708 random bytes less `0` (placed); the word at `c2+0x10` points at 24 random bytes less `0` (placed); the word at `p49+0` points at 16 random bytes less `0` (placed); the word at `p49+4` points at 16 random bytes less `0` (placed); the word at `p49+8` points at 16 random bytes less `0` (placed); the word at `p49+12` points at 16 random bytes less `0` (placed); the word at `p49+16` points at 16 random bytes less `0` (placed); the word at `p49+20` points at 16 random bytes less `0` (placed); the word at `0xc0aec0` points at 12 random bytes less `0` (placed); the word at `p56+0` points at 16 random bytes less `0` (placed); the word at `p56+4` points at 16 random bytes less `0` (placed); the word at `p56+8` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 12 random bytes less `0` (placed); the word at `p60+0` points at 16 random bytes less `0` (placed); the word at `p60+4` points at 16 random bytes less `0` (placed); the word at `p60+8` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `c5+0x134` points at 1024 random bytes less `0` (placed); the word at `0xc09970` points at 12 random bytes less `0` (placed); the word at `0xc0998c` points at 12 random bytes less `0` (placed); the word at `0xc099a8` points at 12 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; slot +0x14 laid as 0x004707c0 or its stub, a flag after the answers; slot +0x80 laid as 0x0046ce10 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 14 answered from [0, 1] (90%); the word at `p36+0` laid as the image vtable at 0xb41fd4 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 0xb41fd4 (the stub slots copied there) half the rows; the word at `p38+0` laid as the image vtable at 0xb41fd4 (the stub slots copied there) half the rows; the word at `p39+0` laid as the image vtable at 0xb41fd4 (the stub slots copied there) half the rows; the word at `p40+0` laid as the image vtable at 0xb41fd4 (the stub slots copied there) half the rows; the word at `p41+0` laid as the image vtable at 0xb41fd4 (the stub slots copied there) half the rows; the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 16 answers 4 values in call order, printed after the answers; stub 16 answers -70000..70000 every row; stub 9 answers 0..6 every row; stub 10 answers 0..6 every row; stub 11 answers 0..6 every row; stub 12 answers 0..6 every row; stub 21 answers -1..8 every row; stub 22 answers -2..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x7e4:0x4, +0x940:0x4, +0x95c:0x4, +0xa48:0x4, +0xe40:0xb0, +0x6c18:0x8 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_finished_628490.py") else {
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
        let log: Vec<String>;
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: m the building (`+8` flags, `+9` owner, `+0xa` id, `+0x10` / `+0x14` the xor-coded
                // tile, `+0x18` the owner object, `+0x72` the city); p[7l..7l+7] the leader l's `+0` flags,
                // `+0x7e4` cap, `+0x940` control, `+0x95c` population gained, `+0xa48` government hero frame,
                // `+0xe40..` its region-population shorts (`+0xe62` is index 17), `+0x6c18` its availability
                // bits; p[22+t] the type record t (`+4` its kind), p[29+t] the unit price record (`+0x2f0`
                // pop), p[36+t] the unit type (`+0x2b4` / `+0x2b8` flags), p[43+t] the build type (`+0x2c0`),
                // p[50+t] the spell record, p[57+4k+j] the unit j of the objects' list k, p[64] the `+0x18`
                // object, p[65] the tile table, p[66+l] leader l's city list; c3 the game (`+0x6cc` world
                // population), c4 the div-by-3 table, c5 the world (`+0` width), c6 the objects (`+0x200` the
                // units list). `a[0]` the type index, `a[1+i]` stub i's answer, `a[24..28]` the four
                // population values, `a[28..32]` whether the type records' slots `+0xc`, `+0x48`, `+0x14` and
                // the units' `+0x80` hold the image function.
                use crate::production::{Handover, finished};
                fn call(log: &mut Vec<String>, s: usize, ecx: &str, args: &[String]) {
                    let mut t = format!("s{s}:{ecx}");
                    for x in args {
                        t.push(',');
                        t.push_str(x);
                    }
                    log.push(t);
                }
                fn i16at(w: &[u8], o: usize) -> i16 {
                    i16::from_le_bytes([w[o], w[o + 1]])
                }
                fn run(
                    a: &[i32],
                    m: &[u8],
                    c: &mut [Vec<u8>],
                    p: &mut [Vec<u8>],
                    log: &mut Vec<String>,
                ) -> i64 {
                    let ti = a[0];
                    let t = ti as usize;
                    let who = m[9] as usize;
                    let lead = format!("p{}+0x0", 7 * who);
                    let this = "m+0x0".to_string();
                    let num = |v: i32| v.to_string();
                    let ans = |s: usize| a[1 + s];
                    let xc = rd(m, 0x10) ^ 0x63637;
                    let yc = rd(m, 0x14) ^ 0x63637;
                    let trec = format!("p{}+0x0", 22 + t);
                    let kind = rd(&p[22 + t], 4);
                    let image_vt = rd(m, 0) as u32 == 0xb42174;
                    // The unit-type arm: the kind is tested through slot +0xc (the image function reads the
                    // range, the stub answers).
                    let is_unit = if a[28] != 0 {
                        (0x32..0x19e).contains(&kind)
                    } else {
                        call(log, 0, &trec, &[]);
                        ans(0) != 0
                    };
                    if is_unit && p[7 * who + 6][t >> 3] & (1 << (t & 7)) != 0 {
                        let cap = rd(&p[7 * who + 1], 0);
                        let sum = rd(&p[7 * who + 2], 0).wrapping_add(rd(&p[29 + t], 0x2f0));
                        // The population test comes first: `cap < control + pop`.
                        if finished(true, cap, sum, 0, false) == Handover::Population {
                            return 0;
                        }
                        let u = &p[36 + t];
                        let urec = format!("p{}+0x0", 36 + t);
                        let caravan = if rd(u, 0) as u32 == 0xb41fd4 {
                            rd(u, 0x2b8) & 8
                        } else {
                            call(log, 3, &urec, &[]);
                            ans(3)
                        };
                        let mut at_limit = false;
                        if caravan != 0 {
                            call(log, 9, &lead, &[num(ti), "0".into()]);
                            call(log, 10, &lead, &["0".into()]);
                            at_limit = ans(9) >= ans(10);
                        }
                        if !at_limit {
                            let air = if image_vt {
                                call(log, 5, "p64+0x0", &["447".into(), "0".into()]);
                                ans(5)
                            } else {
                                call(log, 4, &this, &["447".into(), "0".into()]);
                                ans(4)
                            };
                            if air != 0 && rd(u, 0x2b4) & 0x20 == 0 {
                                call(log, 11, &this, &["0".into()]);
                                call(log, 12, &this, &[]);
                                at_limit = ans(11) >= ans(12);
                            }
                        }
                        if finished(true, cap, sum, 0, at_limit) == Handover::Limit {
                            return -1;
                        }
                        call(log, 13, &this, &[num(ti)]);
                        return 1;
                    }
                    // The spell arm.
                    let is_spell = if a[29] != 0 {
                        (0x275..0x2ac).contains(&kind)
                    } else {
                        call(log, 1, &trec, &[]);
                        ans(1) != 0
                    };
                    if is_spell {
                        call(log, 14, &lead, &[num(ti)]);
                        if ans(14) != 0 {
                            let id = i32::from(i16at(m, 0xa));
                            call(
                                log,
                                15,
                                &format!("p{}+0x0", 50 + t),
                                &[num(id), num(i32::from(m[9])), "0".into(), "0".into()],
                            );
                            return 1;
                        }
                    }
                    // The building arm.
                    let is_build = if a[30] != 0 {
                        (0x19e..0x21f).contains(&kind)
                    } else {
                        call(log, 2, &trec, &[]);
                        ans(2) != 0
                    };
                    if is_build && rd(&p[43 + t], 0x2c0) & 4 == 0 {
                        let city = i16at(m, 0x72) as isize;
                        let cw = rd(&p[66 + who], (4 * city) as usize);
                        let city_tok = cw.to_string();
                        let mut pops = a[24..28].iter();
                        call(log, 16, &city_tok, &[]);
                        let pop1 = *pops.next().unwrap();
                        call(log, 6, &this, &[num(ti), "0".into()]);
                        call(log, 17, &this, &["1".into(), "0".into()]);
                        let f = rd(&p[7 * who], 0);
                        wr(&mut p[7 * who], 0, f | 0x8000000);
                        if m[8] & 0x20 == 0 {
                            return 1;
                        }
                        call(log, 18, &city_tok, &[]);
                        call(log, 16, &city_tok, &[]);
                        let pop2 = *pops.next().unwrap();
                        let g = rd(&p[7 * who + 3], 0);
                        wr(
                            &mut p[7 * who + 3],
                            0,
                            g.wrapping_add(pop2.wrapping_sub(pop1)),
                        );
                        call(log, 16, &city_tok, &[]);
                        let pop3 = *pops.next().unwrap();
                        let wp = rd(&c[3], 0x6cc);
                        wr(&mut c[3], 0x6cc, wp.wrapping_add(pop3.wrapping_sub(pop1)));
                        let ix = rd(&c[4], 4 * (xc >> 8) as usize);
                        let iy = rd(&c[4], 4 * (yc >> 8) as usize);
                        let tile = (iy.wrapping_mul(rd(&c[5], 0)).wrapping_add(ix)) as usize;
                        let region = i32::from(i16at(&p[65], tile * 0x1c + 4));
                        if region < 0x40 {
                            call(log, 16, &city_tok, &[]);
                            let pop4 = *pops.next().unwrap();
                            let off = (0xe62 - 0xe40 + 2 * region) as usize;
                            let s = &mut p[7 * who + 5];
                            let v = i16at(s, off).wrapping_add(pop4.wrapping_sub(pop1) as i16);
                            s[off..off + 2].copy_from_slice(&v.to_le_bytes());
                        }
                        call(log, 19, &lead, &[]);
                        call(log, 20, &lead, &[]);
                        return 1;
                    }
                    // The tail: the research completes through `gain_tech`.
                    call(
                        log,
                        8,
                        &lead,
                        &[num(ti), num(xc), num(yc), "1".into(), "1".into()],
                    );
                    let senate = if image_vt {
                        call(log, 5, "p64+0x0", &["438".into(), "0".into()]);
                        ans(5)
                    } else {
                        call(log, 4, &this, &["438".into(), "0".into()]);
                        ans(4)
                    };
                    if senate != 0 {
                        call(log, 21, &lead, std::slice::from_ref(&lead));
                        let hero = ans(21);
                        if hero >= 0 {
                            if rd(&p[7 * who + 4], 0) < 0 {
                                call(log, 13, &this, &[num(hero)]);
                                return 1;
                            }
                            let k = rd(&c[6], 0x200) as usize;
                            let args: Vec<String> = [
                                xc,
                                yc,
                                1,
                                i32::from(m[9]),
                                -1,
                                0x20000,
                                0x17,
                                0,
                                0,
                                0,
                                0,
                                0,
                                yc,
                            ]
                            .iter()
                            .map(|v| v.to_string())
                            .collect();
                            call(log, 22, "c6+0x0", &args);
                            let j = ans(22);
                            if j >= 0 {
                                let ecx = format!("p{}+0x0", 57 + 4 * k + j as usize);
                                if a[31] != 0 {
                                    call(log, 6, &ecx, &[num(hero), "0".into()]);
                                } else {
                                    call(log, 7, &ecx, &[num(hero), "0".into()]);
                                }
                            }
                        }
                    }
                    1
                }
                let mut l = Vec::new();
                let r = run(a, m, c, p, &mut l);
                log = if l.is_empty() {
                    vec!["-".to_string()]
                } else {
                    l
                };
                r
            }
        };
        assert_eq!(got, want_eax, "finished — eax, row `{line}`");
        assert_eq!(log, want_log, "finished — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "finished — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "finished — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "finished — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Build::finished@00628490: {n} rows agree");
}
