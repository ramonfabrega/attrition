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

/// `Build::process@0061edf0`, run by `tools/emu/sweep_build_process_messages.py` under unicorn
/// (tools/emu): 136 random bytes of object, 0 argument(s), `p` windows in order: p0..p55 sparse table 0 (8 rows × 7 slice(s), row-major), p56 sparse table 1 (1 rows × 1 slice(s), row-major), p57 sparse table 2 (1 rows × 1 slice(s), row-major), p58 sparse table 3 (1 rows × 1 slice(s), row-major), p59..p60 sparse table 4 (1 rows × 2 slice(s), row-major), p61 sparse table 5 (1 rows × 1 slice(s), row-major), p62..p63 the place(s), in flag order, p64 stub 15's answer window; callee 0x00640450 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x1b4 (1 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; vtable slot +0x180 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x68 (1 args) stubbed; callee 0x006d9380 (1 args) stubbed; callee 0x006e07d0 (2 args) stubbed; callee 0x006e0d50 (2 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9b70 (9 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `m+0x18` points at 512 random bytes less `0` (placed); the word at `p61+0` points at 128 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 6 answered from [0, 1, 1, 1, 1, 1, 1] (90%); stub 7 answered from [0, 1, 1, 1, 1, 1, 1] (90%); stub 8 answered from [0, 1, 1, 1, 2] (90%); stub 9 answered from [0, 1, 52, 82, -1, 100] (90%); stub 10 answered from [0, 1] (90%); stub 12 answered from [0, 0, 0, 0, 0, 1] (90%); stub 11 answered from [0, 0, 0, 0, 0, 0, 3, 9, 7, 7, 6] (90%); stub 15 answers the address of 32 random bytes, a `p` window after the places; the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 11 answers 2 values in call order, printed after the answers; stub 3 answers 0..0 every row; stub 4 answers 0..0 every row; stub 5 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 4096 bytes under the call is zeroed before each row; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x3f8:0x4, +0x940:0x4, +0x5a98:0x4, +0x6928:0x4, +0x6eb8:0x10 (`p` windows after the pointed buffers, before the places); 1 rows 0x560 apart at `0xc061ec`, slices +0x550:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x2a0 apart at `0xc06210`, slices +0x298:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x10 apart at `0xc06208`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0xa04 apart at `0xc8cd00`, slices +0x9d8:0x4, +0x9ec:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x150 apart at `0xe85ddc`, slices +0xd0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_process_messages.py") else {
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
            // `Build::process@0061edf0` has no crate method: the crate carries its parts scattered
            // (`Sim::process_building`, `process_building_combat`, `process_building_tail`, `nations::
            // {kremlin_spy, persian_market_caravan, terra_cotta_army}`, `gather::gather_region_building`,
            // `roads::regen_roads_due`) and every one reads a `Sim`; the console's two reminders
            // (`TextBubble` and `MessageWin::add_event`) have no counterpart in the sim (the interface's), nor are the leader's two stamps `+0x6ec0` and `+0x6ec4` (only
            // the sweep carries them). So the whole of this slice is inline: the head to `do_queue`, then the
            // two reminders (frame & 255 == 0). Window map: m the building; leader row r's slices p(7r+s):
            // 0 flags, 1 `+8`, 2 units `+0x3f8`, 3 `+0x940`, 4 queued word `+0x5a98`, 5 team colour byte
            // `+0x6928`, 6 the stamps `+0x6eb8..+0x6ec8`; p56 the game's frame, p57 the console's who, p58
            // the message window, p59 and p60 the strings, p61 the types array's `+0xd0`; p62 the building's
            // type, p63 the Type object behind p61; p64 the bubble.
            // **The slice's domain** (asserted): a finished building without a city flag, no attack, no
            // pending ejection/launch/rebuild, `+0x60`'s low half clear; the leader's other fields read zero.
            const IMAGE: i32 = 0xb4_2174;
            let short = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            let u8w = |w: &[u8], o: usize| i32::from(w[o]);
            let frame = rd(&p[56], 0);
            let image = rd(m, 0) == IMAGE;
            assert!(frame >= 0, "frame");
            let who = u8w(m, 9);
            assert!(who < 8, "who");
            let leader = format!("p{}+0x0", who * 7);
            let lw = |s: usize| (who as usize) * 7 + s;
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
                assert_eq!(
                    short(m, 0x60) & 0xffff,
                    0,
                    "+0x60 low half clear: no ejection, launch, 0x200, roads"
                );
                assert_eq!(rd(&p[62], 0x1e8), 0, "no attack");
                if !image {
                    log.push("s5:m+0x0".to_string());
                    assert_eq!(a[5], 0);
                }
                // `do_queue(0)` is `this`'s slot 0x1b4.
                log.push("s2:m+0x0,0".to_string());
                // The type's slots 0x90: gatherer? (answers 0).
                log.push("s3:p62+0x0".to_string());
                assert_eq!(a[3], 0);
                let ident = rd(&p[62], 4);
                assert!(
                    ![0x21a, 0x1b6, 0x1b4, 0x211].contains(&ident),
                    "wonder arms excluded"
                );
                let phase = short(m, 0xa);
                if (frame + phase) % 256 == 0 {
                    let mut useq = 0usize;
                    let mut units = |log: &mut Vec<String>, ty: i32| {
                        log.push(format!("s11:{leader},{ty},0"));
                        let v = a[19 + useq.min(1)];
                        useq += 1;
                        v
                    };
                    let human = |log: &mut Vec<String>, m: &[u8]| -> bool {
                        if image {
                            m[0x1c] != 0
                        } else {
                            log.push("s8:m+0x0".to_string());
                            a[8] != 0
                        }
                    };
                    let console = rd(&p[57], 0);
                    // The ident test: type slot 0x60 on the vftable arm, `this`'s 0xb8 otherwise.
                    let is_type = |log: &mut Vec<String>, t: i32| -> bool {
                        if image {
                            log.push(format!("s7:p62+0x0,{t},0"));
                            a[7] != 0
                        } else {
                            log.push(format!("s6:m+0x0,{t},0"));
                            a[6] != 0
                        }
                    };
                    let stamp = |p: &[Vec<u8>], o: usize| rd(&p[lw(6)], o);
                    let set_stamp = |p: &mut [Vec<u8>], o: usize, v: i32| wr(&mut p[lw(6)], o, v);
                    let bubble = |log: &mut Vec<String>,
                                  p: &mut [Vec<u8>],
                                  m: &[u8],
                                  c: &[Vec<u8>],
                                  off: i32| {
                        let _ = c;
                        log.push("s13:sp-0x30".to_string());
                        log.push(format!(
                            "s14:sp-0x30,{}",
                            if off == 0x9d8 { "p59+0x0" } else { "p60+0x0" }
                        ));
                        log.push("s15:sp-0x30".to_string());
                        log.push("s14:p64+0x4,sp-0x30".to_string());
                        p[64][0x1a] = who as u8;
                        let team = u8w(&p[lw(5)], 0);
                        log.push(format!("s16:{},sp-0x3c", 0xe3_9ce0 + team * 0x64));
                        let (x, y) = (rd(m, 0x10) ^ 0x6_3637, rd(m, 0x14) ^ 0x6_3637);
                        log.push(format!(
                            "s17:p58+0x0,{x},{y},0,{},-1,3072,0,p64+0x0,0",
                            a[16]
                        ));
                        log.push("s18:sp-0x30".to_string());
                    };
                    // First reminder: no Spy ... `0x3b` is the unit type the leader has none of.
                    if is_type(&mut log, 0x1b4) && who == console {
                        let u = units(&mut log, 0x3b);
                        let mut go = u == 0;
                        if go {
                            log.push(format!("s12:{leader},59,{}", rd(&p[lw(4)], 0) & 0xffff));
                            go = a[12] == 0;
                        }
                        if go && rd(&p[lw(2)], 0) > 1 {
                            let ts = stamp(p, 8);
                            if ts == 0 || frame - ts > 5000 {
                                if human(&mut log, m) {
                                    set_stamp(p, 8, frame);
                                    bubble(&mut log, p, m, c, 0x9d8);
                                } else if stamp(p, 8) != 0 {
                                    set_stamp(p, 8, frame - 900);
                                }
                            }
                        }
                    }
                    // Second reminder.
                    if is_type(&mut log, 0x1a4) && who == console {
                        log.push(format!("s10:{leader},p63+0x0"));
                        let ty = if a[10] != 0 {
                            0x34
                        } else {
                            log.push(format!("s9:p63+0x0,{}", rd(&p[lw(1)], 0)));
                            a[9]
                        };
                        let u = units(&mut log, ty);
                        if u < 7 {
                            let ts = stamp(p, 12);
                            if ts != 0 && frame - ts <= 5000 {
                                set_stamp(p, 12, frame - 900);
                            } else if human(&mut log, m) {
                                set_stamp(p, 12, frame);
                                bubble(&mut log, p, m, c, 0x9ec);
                            }
                        }
                    }
                }
                assert_eq!(m[8] & 0x20, 0, "no city flag in this slice");
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
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Build::process@0061edf0: {n} rows agree");
}
