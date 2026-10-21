//! `Build::process@0061edf0` under the emulator against the port (item 3403; tools/sweep_gen.py, obj with windows).

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

/// `Build::process@0061edf0`, run by `tools/emu/sweep_build_process_wonders.py` under unicorn
/// (tools/emu): 136 random bytes of object, 0 argument(s), `p` windows in order: p0..p17 sparse table 0 (2 rows × 9 slice(s), row-major), p18..p19 sparse table 1 (1 rows × 2 slice(s), row-major), p20..p22 sparse table 2 (1 rows × 3 slice(s), row-major), p23 sparse table 3 (1 rows × 1 slice(s), row-major), p24 sparse table 4 (1 rows × 1 slice(s), row-major), p25..p33 the place(s), in flag order; callee 0x00640450 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x1b4 (1 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x180 (0 args) stubbed; callee 0x0062f9b0 (1 args) stubbed; callee 0x006e0600 (1 args) stubbed; callee 0x006e0680 (2 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006dca50 (1 args) stubbed; callee 0x006e1330 (1 args) stubbed; callee 0x006d9380 (1 args) stubbed; vtable slot +0x68 (1 args) stubbed; callee 0x006e3140 (1 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p25+0` points at 112 random bytes less `0` (placed); the word at `p25+4` points at 112 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p28+0` points at 112 random bytes less `0` (placed); the word at `p28+4` points at 112 random bytes less `0` (placed); the word at `m+0x18` points at 496 random bytes less `0` (placed); the word at `p23+0` points at 760 random bytes less `0` (placed); the word at `p24+0` points at 128 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 8 answered from [0, 1, 1] (90%); stub 10 answered from [0, 0, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 5, 82] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p26+0x18` laid as the address of `p32` every row; the word at `p27+0x18` laid as the address of `p32` every row; the word at `p29+0x18` laid as the address of `p32` every row; the word at `p30+0x18` laid as the address of `p32` every row; stub 3 answers 0..0 every row; stub 4 answers 0..0 every row; stub 5 answers -1..1 every row; stub 6 answers -1..3 every row; stub 7 answers -2..2 every row; stub 9 answers 0..5 every row; stub 13 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 4096 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x7e4:0x4, +0x940:0x4, +0x980:0x4, +0x9f8:0x4, +0xa48:0x4, +0x5222:0x10, +0x558e:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x6d0 apart at `0xc061ec`, slices +0x550:0x4, +0x6c8:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x520 apart at `0xc061f0`, slices +0x464:0x4, +0x468:0x4, +0x514:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x6e0 apart at `0xc0aaa0`, slices +0x6d8:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x150 apart at `0xe85ddc`, slices +0x148:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_process_wonders.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `Build::process@0061edf0`'s wonders: the Kremlin's spy, the government hero, the Persian
            // Market's caravan and the Terra Cotta Army's soldier. The crate carries them as
            // `Sim::kremlin_spy`, `persian_market_caravan` and `terra_cotta_army` (`nations.rs`), each
            // over a `Sim` and a typed `Tech`; the government hero (`+0xa48 == frame + 1`) is not modelled
            // at all. So the slice is inline. Window map: leader row r (0, 1) slices p(9r+s): 0 flags, 1 `+8`,
            // 2 `+0x7e4`, 3 `+0x940` (control), 4 `+0x980` (caravans), 5 `+0x9f8`, 6 `+0xa48`, 7 `+0x5222`
            // (sixteen bytes of words), 8 `+0x558e`; p18 the game's frame, p19 its `terra_cotta_total`;
            // p20 `TERRA_COTTA_FREE`, p21 `TERRA_COTTA_DELAY`, p22 `KREMLIN_SPY`; units of player K: array
            // p25+3K, records p26+3K+i; p31 the building's type; p32 the build-type record (`+0x2e0` the
            // government building's index, `+0x2f0` the spy's cost behind every unit's type pointer);
            // p33 the Type object `types[0x148]`. **Domain** (asserted): a finished building without a city
            // index, `+0x60`'s low half clear, no attack, the first reminder's gate shut.
            const IMAGE: i32 = 0xb4_2174;
            let short = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let frame = rd(&p[18], 0);
            let image = rd(m, 0) == IMAGE;
            let who = usize::from(m[9]);
            assert!(frame >= 0 && who < 2, "frame, who");
            let lw = |s: usize| 9 * who + s;
            let leader = format!("p{}+0x0", 9 * who);
            log.push("s0:m+0x0".to_string());
            let active = if image {
                m[8] & 4 != 0
            } else {
                log.push("s1:m+0x0".to_string());
                a[1] != 0
            };
            if active {
                if short(m, 0x38) != 0 {
                    let v = (short(m, 0x38) - 1) as i16;
                    m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
                }
                assert_eq!(short(m, 0x60), 0, "+0x60's low half is clear");
                assert_eq!(rd(&p[31], 0x1e8), 0, "no attack");
                assert!(short(m, 0x72) < 0, "no city index");
                if !image {
                    log.push("s4:m+0x0".to_string());
                    assert_eq!(a[4], 0);
                }
                log.push("s2:m+0x0,0".to_string());
                log.push("s3:p31+0x0".to_string());
                assert_eq!(a[3], 0);
                let ident = rd(&p[31], 4);
                let x = frame + short(m, 0xa);
                // The Kremlin's spy.
                if ident == 0x21a && x % 15 == 0 && rd(&p[lw(0)], 0) & 0x1000_0000 != 0 {
                    let v = u16at(m, 0x7a).wrapping_sub(1);
                    m[0x7a..0x7c].copy_from_slice(&v.to_le_bytes());
                    if (v as i16) <= 0 {
                        let f = rd(&p[lw(0)], 0) & !0x1000_0000;
                        wr(&mut p[lw(0)], 0, f);
                        let k = (rd(&p[22], 0) & 0xffff) as u16;
                        m[0x7a..0x7c].copy_from_slice(&k.to_le_bytes());
                        log.push("s5:m+0x0,58".to_string());
                        let r = a[5];
                        if r >= 0 {
                            let u = 26 + 3 * who + r as usize;
                            let fl = rd(&p[u], 0x6c) | 0x400;
                            wr(&mut p[u], 0x6c, fl);
                            let cost = if p[u][0x68] & 1 == 0 {
                                rd(&p[32], 0x2f0)
                            } else {
                                0
                            };
                            let c = rd(&p[lw(3)], 0).wrapping_sub(cost);
                            wr(&mut p[lw(3)], 0, c);
                        }
                    }
                }
                // The government hero.
                if rd(&p[lw(6)], 0) == frame + 1 {
                    if ident == 0x1b6 {
                        log.push(format!("s6:{leader},{leader}"));
                        if a[6] >= 0 {
                            log.push(format!("s5:m+0x0,{}", a[6]));
                        }
                    }
                    if m[8] & 0x20 != 0 {
                        let mut ebx = i32::from(u16at(&p[lw(8)], 0));
                        let ecx = rd(&p[32], 0x2e0);
                        if ecx >= 0 {
                            let word = u16at(&p[lw(7)], 2 * ecx as usize);
                            log.push(format!("s7:{leader},{ecx},{word}"));
                            ebx = ebx.wrapping_add(a[7]);
                        }
                        if ebx <= 0 {
                            let arg = if ecx >= 0 {
                                leader.clone()
                            } else {
                                ecx.to_string()
                            };
                            log.push(format!("s6:{leader},{arg}"));
                            if a[6] >= 0 {
                                log.push(format!("s5:m+0x0,{}", a[6]));
                            }
                        }
                    }
                }
                // The Persian Market's caravan.
                if ident == 0x1b4 {
                    log.push(format!("s8:{leader},23"));
                    if a[8] != 0 && x % 15 == 0 {
                        log.push(format!("s9:{leader},1"));
                        if rd(&p[lw(4)], 0) < a[9] {
                            log.push(format!("s10:{leader},59"));
                            if a[10] == 0 {
                                log.push("s5:m+0x0,59".to_string());
                            }
                        }
                    }
                }
                // The Terra Cotta Army's soldier.
                if ident == 0x211 {
                    let v = u16at(m, 0x7a).wrapping_sub(1);
                    m[0x7a..0x7c].copy_from_slice(&v.to_le_bytes());
                    if (v as i16) <= 0 {
                        let free = rd(&p[20], 0) as i16;
                        let t = (free << 4).wrapping_sub(free);
                        let own = rd(&p[lw(5)], 0);
                        let delay = rd(&p[21], 0);
                        if delay == 7 {
                            let e = own.wrapping_shl(4).wrapping_sub(own);
                            let h = (e - (e >> 31)) >> 1;
                            let k = h.wrapping_add(i32::from(t as u16)) as u16;
                            m[0x7a..0x7c].copy_from_slice(&k.to_le_bytes());
                        } else {
                            let prod = (own as i16 as i32).wrapping_mul(delay as i16 as i32) as i16;
                            let k = prod.wrapping_add(t) as u16;
                            m[0x7a..0x7c].copy_from_slice(&k.to_le_bytes());
                        }
                        log.push(format!("s11:{leader},p33+0x0"));
                        let ty = if a[11] != 0 {
                            0x52
                        } else {
                            log.push(format!("s12:p33+0x0,{}", rd(&p[lw(1)], 0)));
                            a[12]
                        };
                        log.push(format!("s13:{leader},{ty}"));
                        let up = a[13];
                        if up >= 0 && rd(&p[lw(3)], 0) < rd(&p[lw(2)], 0) {
                            log.push(format!("s5:m+0x0,{up}"));
                            let n = rd(&p[19], 0).wrapping_add(1);
                            wr(&mut p[19], 0, n);
                        }
                    }
                }
                assert_ne!(x % 256, 0, "the first reminder's gate is shut");
            }
            0i64
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Build::process@0061edf0: {n} rows agree");
}
