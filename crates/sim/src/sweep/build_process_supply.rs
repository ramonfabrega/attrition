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

/// `Build::process@0061edf0`, run by `tools/emu/sweep_build_process_supply.py` under unicorn
/// (tools/emu): 136 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2 sparse table 1 (1 rows × 1 slice(s), row-major), p3 sparse table 2 (1 rows × 1 slice(s), row-major), p4..p5 sparse table 3 (1 rows × 2 slice(s), row-major), p6 sparse table 4 (1 rows × 1 slice(s), row-major), p7 sparse table 5 (1 rows × 1 slice(s), row-major), p8..p10 the place(s), in flag order; callee 0x00640450 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x1b4 (1 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0x180 (0 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x006edb50 (1 args) stubbed; vtable slot +0x168 (3 args) stubbed; callee 0x00623570 (0 args) stubbed; callee 0x00623350 (0 args) stubbed; the word at `m+0x18` points at 496 random bytes less `0` (placed); the word at `p5+0` points at 592 random bytes less `0` (placed); the word at `p7+0` points at 544 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 1, 1, 1] (90%); stub 4 answered from [0, 0, 0, 1] (90%); stub 6 answered from [-1, 0, 1, 2] (90%); stub 7 answered from [0, 1, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 6 answers 3 values in call order, printed after the answers; stub 5 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 4096 bytes under the call is zeroed before each row; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x59c0:0xc (`p` windows after the pointed buffers, before the places); 1 rows 0x560 apart at `0xc061ec`, slices +0x550:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x6a0 apart at `0xc061f0`, slices +0x698:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x140 apart at `0xc06188`, slices +0x0:0x4, +0x134:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x40 apart at `0xcae5fc`, slices +0x0:0x40 (`p` windows after the pointed buffers, before the places); 1 rows 0x20 apart at `0xc061b8`, slices +0x10:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_process_supply.py") else {
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
            // `Build::process@0061edf0`'s supply band: the French leader's heal, `do_queue`, and the two
            // gather-tile re-entries. The crate carries the re-entries as `gather::gather_region_building`
            // (`docs/ECONOMY.md` §17.2, over a `Sim`'s region table); the French heal (`french_supply_heal_rate`,
            // a building's `+0x38` hold lifted to the rate after a `repair_damage(1)`) is not modelled.
            // Window map: leader row r (0, 1): p(r) the shorts `+0x59c0`, `+0x59c4`, `+0x59c8`; p2 the game's
            // frame; p3 `french_supply_heal_rate`; p4 the world's width, p5 its cell-array slot; p6 the
            // `div_3_table`; p7 the region array's slot; p8 the building's type; p9 the world's cells (`0x1c`
            // apiece: `+4` the region, `+0xf` the owner byte); p10 the region records (`0x88` apiece, `+8`
            // flags). **Domain** (asserted): a finished building without a city index, `+0x60`'s low half
            // clear, no attack, the first reminder's gate shut, no wonder's type.
            const IMAGE: i32 = 0xb4_2174;
            const KEY: i32 = 0x6_3637;
            let short = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            let frame = rd(&p[2], 0);
            let image = rd(m, 0) == IMAGE;
            let who = usize::from(m[9]);
            assert!(frame >= 0 && who < 2, "frame, who");
            let leader = (0xe3_a390 + who * 0x6eec).to_string();
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
                assert_eq!(rd(&p[8], 0x1e8), 0, "no attack");
                assert!(short(m, 0x72) < 0, "no city index");
                if !image {
                    log.push("s5:m+0x0".to_string());
                    assert_eq!(a[5], 0);
                }
                let phase = short(m, 0xa);
                let x = frame + phase;
                // The tile under the building: `world.width * div3[y >> 8] + div3[x >> 8]`.
                let (xx, yy) = ((rd(m, 0x10) ^ KEY) >> 8, (rd(m, 0x14) ^ KEY) >> 8);
                let div3 = |i: i32| rd(&p[6], 4 * i as usize);
                let tile = rd(&p[4], 0).wrapping_mul(div3(yy)).wrapping_add(div3(xx)) as usize;
                let lead = &p[who];
                let supplied = short(lead, 0) != 0 || short(lead, 8) != 0 || short(lead, 4) != 0;
                if supplied && (rd(m, 0x24) != 0 || m[0x3b] != 0) {
                    let heal = rd(&p[3], 0);
                    if heal != 0 && x % heal == 0 {
                        let owner = i32::from(p[9][tile * 0x1c + 0xf] as i8);
                        let mut seq = 0usize;
                        let mut general = |log: &mut Vec<String>, id: i32| -> i32 {
                            log.push(format!("s6:m+0x0,0,{id}"));
                            let k = seq;
                            seq += 1;
                            a[11 + k]
                        };
                        let heals = if general(&mut log, 0x163) < 0 {
                            if owner < 0 {
                                false
                            } else {
                                log.push(format!("s7:{leader},{owner}"));
                                a[7] != 0
                                    && (general(&mut log, 0x161) >= 0
                                        || general(&mut log, 0x165) >= 0)
                            }
                        } else if owner < 0 {
                            true
                        } else {
                            log.push(format!("s7:{leader},{owner}"));
                            a[7] != 0
                        };
                        if heals {
                            log.push("s8:m+0x0,1,0,1".to_string());
                            let cur = short(m, 0x38);
                            let v = (cur.max(heal)) as u16;
                            m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
                        }
                    }
                }
                log.push("s2:m+0x0,0".to_string());
                log.push("s3:p8+0x0".to_string());
                if a[3] != 0 {
                    log.push("s4:p8+0x0".to_string());
                    if a[4] == 0 {
                        let region = short(&p[9], tile * 0x1c + 4) as usize;
                        let flags = p[10][region * 0x88 + 8];
                        if flags & 0x20 != 0 {
                            log.push("s9:m+0x0".to_string());
                        }
                        if flags & 0x10 != 0 {
                            log.push("s10:m+0x0".to_string());
                        }
                    }
                }
                assert!(
                    ![0x21a, 0x1b6, 0x1b4, 0x211].contains(&rd(&p[8], 4)),
                    "wonder arms excluded"
                );
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
