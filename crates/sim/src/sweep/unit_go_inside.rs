//! `Unit::go_inside@0061a2e0` under the emulator against the port (item 3239; tools/sweep_gen.py, obj with windows).

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

/// `Unit::go_inside@0061a2e0`, run by `tools/emu/sweep_unit_go_inside.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0..p2 sparse table 0 (3 rows × 1 slice(s), row-major), p3..p27 the place(s), in flag order; the global cell 0x00c0618c points at 96 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00e8d444 points at 5088 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 32 random bytes at 0x00add000; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; callee 0x005f96b0 (4 args) stubbed; callee 0x00647e90 (2 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x00401010 (4 args) stubbed; callee 0x00401020 (3 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x00616f40 (3 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p3+12` points at 32 random bytes less `0` (placed); the word at `p3+16` points at 32 random bytes less `0` (placed); the word at `p3+20` points at 32 random bytes less `0` (placed); the word at `m+0xf8` points at 24 random bytes less `0` (placed); the word at `p10+0` points at 256 random bytes less `0` (placed); the word at `p10+4` points at 256 random bytes less `0` (placed); the word at `p10+8` points at 256 random bytes less `0` (placed); the word at `p10+12` points at 256 random bytes less `0` (placed); the word at `p10+16` points at 256 random bytes less `0` (placed); the word at `p10+20` points at 256 random bytes less `0` (placed); the word at `m+0xf4` points at 12 random bytes less `0` (placed); the word at `p17+0` points at 212 random bytes less `0` (placed); the word at `p17+4` points at 212 random bytes less `0` (placed); the word at `p17+8` points at 212 random bytes less `0` (placed); the word at `m+0x18` points at 800 random bytes less `0` (placed); the word at `p11+0x18` points at 800 random bytes less `0` (placed); the word at `p12+0x18` points at 800 random bytes less `0` (placed); the word at `p13+0x18` points at 800 random bytes less `0` (placed); the word at `p14+0x18` points at 800 random bytes less `0` (placed); the word at `p15+0x18` points at 800 random bytes less `0` (placed); the word at `p16+0x18` points at 800 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xc0 laid as 0x0046ce40 or its stub, a flag after the answers; stub 0 answered from [1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); the word at `c0+0x30` laid as the address of `p3` every row; the word at `c0+0x4c` laid as the address of `p3` every row; the word at `p11+0xf4` laid as the address of `p17` every row; the word at `p12+0xf4` laid as the address of `p17` every row; the word at `p13+0xf4` laid as the address of `p17` every row; the word at `p14+0xf4` laid as the address of `p17` every row; the word at `p15+0xf4` laid as the address of `p17` every row; the word at `p16+0xf4` laid as the address of `p17` every row; stub 3 answers `p11+j`, the data of the object its `ecx` is in `p4+j` (j < 6), linked at `+0x8e`; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 12 leaves `ecx` as a callee would, a value from [0, 7, 2147483632, 268435456, 1], after the other numbers on the row; 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_go_inside.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `Unit::go_inside(this, p1, p2, p3)`: byte-window port (the crate's `Sim::go_inside`, garrison.rs,
                // is a whole-Sim mutator and cannot take these windows, so the original's body is spelled out).
                // Windows: `m` this; `p[11 + j]` the data record `objects[owner][j]` answers to slot +0xa8 (so
                // `this` is `cur` = 0 for `m`, `1 + j` for it); `p[4 + j]` the object records themselves, shared by
                // every owner's table; `p[21]` / `p[22 + j]` the type records (+0x18); `p[18..21]` the shared
                // `+0xf4` array's records; `p[0..3]` the leaders rows; `c[1]` the console (`+0x2a0` play), `c[2]` the
                // groups (0x9f0 apart), `c[3]` the div table. Stubs: s0 is_captain, s1 is_plane, s2 the object's
                // slot +0x18, s3 slot +0xa8, s4 update_ceo_position, s5 insert_inside, s6 GroupData::member, s7/s8
                // the group's slots +0x10 / +0x18, s9 close_orders, s10 clear_partial_path, s11 update_action, s12
                // set_new_location, s13 set_angle, s14 set_anim. `a[3 + i]` is s(i)'s answer; s1, s2, s6 answer
                // `a[18..26]`, `a[26..34]`, `a[34..42]` in call order; `a[42]` / `a[43]` lay the image's is_captain /
                // is_plane at slots +0xe8 / +0xc0; `a[44]` is the `ecx` s12 leaves.
                fn sh(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                fn win<'w>(m: &'w mut [u8], p: &'w mut [Vec<u8>], cur: usize) -> &'w mut [u8] {
                    if cur == 0 { m } else { &mut p[10 + cur] }
                }
                fn tok(cur: usize) -> String {
                    if cur == 0 {
                        "m+0x0".to_string()
                    } else {
                        format!("p{}+0x0", 10 + cur)
                    }
                }
                #[allow(clippy::too_many_arguments)]
                fn go(
                    a: &[i32],
                    m: &mut [u8],
                    c: &mut [Vec<u8>],
                    p: &mut [Vec<u8>],
                    log: &mut Vec<String>,
                    n: &mut [usize; 3],
                    mut cur: usize,
                    p1: i32,
                    p2: i32,
                    p3: i32,
                ) {
                    let (keep_e8, keep_c0) = (a[42] != 0, a[43] != 0);
                    loop {
                        let w = win(m, p, cur);
                        if rd(w, 0x6c) & 0x10000 != 0 {
                            let x = rd(&c[3], 4 * ((rd(w, 0x10) ^ 0x63637) >> 6) as usize);
                            let y = rd(&c[3], 4 * ((rd(w, 0x14) ^ 0x63637) >> 6) as usize);
                            log.push(format!("s4:{},{x},{y},-1,-1", tok(cur)));
                        }
                        if p3 != 0 {
                            break;
                        }
                        let w = win(m, p, cur);
                        let captain = if keep_e8 {
                            i32::from(u16::from_le_bytes([w[0x8e], w[0x8f]]) >> 15)
                        } else {
                            log.push(format!("s0:{}", tok(cur)));
                            a[3]
                        };
                        if captain != 0 {
                            break;
                        }
                        let idx = usize::try_from(sh(w, 0x8e)).expect("captain index");
                        log.push(format!("s3:p{}+0x0", 4 + idx));
                        cur = 1 + idx;
                    }
                    let tw = if cur == 0 { 21 } else { 21 + cur };
                    let (t4, t218, t2b4, t308) = (
                        rd(&p[tw], 4),
                        rd(&p[tw], 0x218),
                        p[tw][0x2b4],
                        rd(&p[tw], 0x308),
                    );
                    log.push(format!("s5:{},{p1},{p2}", tok(cur)));
                    let skip = if keep_c0 {
                        t218 == 2 && t2b4 & 0x20 == 0
                    } else {
                        log.push(format!("s1:{}", tok(cur)));
                        let v = a[18 + n[0].min(7)];
                        n[0] += 1;
                        v != 0
                    };
                    if !skip {
                        let w = win(m, p, cur);
                        let (short, owner) = (sh(w, 0xa), i32::from(w[9]));
                        let play = usize::try_from(rd(&c[1], 0x2a0)).expect("play");
                        let g = format!("c2+{:#x}", play * 0x9f0);
                        log.push(format!("s6:{g},{short},{owner},1"));
                        let ans = a[34 + n[2].min(7)];
                        n[2] += 1;
                        if ans != 0 {
                            log.push(format!("s7:{g},{short},{owner},0,0"));
                            if rd(&c[2], play * 0x9f0 + 0xc) == 0 {
                                log.push(format!("s8:{g},{p1},{p2},0"));
                            }
                        }
                    }
                    let bj = usize::try_from(p1).expect("building index");
                    log.push(format!("s2:p{}+0x0", 4 + bj));
                    let ans = a[26 + n[1].min(7)];
                    n[1] += 1;
                    if ans != 0 && t308 > 1 {
                        let w = win(m, p, cur);
                        let f = rd(w, 0x68) & 0xfbff_ffffu32 as i32;
                        wr(w, 0x68, f);
                        wr(w, 0xc0, 0);
                        log.push(format!("s9:{},0", tok(cur)));
                        log.push(format!("s10:{}", tok(cur)));
                        log.push(format!("s11:{}", tok(cur)));
                    }
                    let i90 = sh(win(m, p, cur), 0x90);
                    if i90 >= 0 {
                        let k = usize::try_from(i90).expect("index");
                        log.push(format!("s3:p{}+0x0", 4 + k));
                        if p[11 + k][8] & 1 != 0 {
                            log.push(format!("s3:p{}+0x0", 4 + k));
                            go(a, m, c, p, log, n, 1 + k, p1, p2, 1);
                        }
                    }
                    if t4 == 0x34 || t4 == 0x35 {
                        let owner = usize::from(win(m, p, cur)[9]);
                        let v = rd(&p[owner], 0) | 0x0200_0000;
                        wr(&mut p[owner], 0, v);
                        let (x, y) = (
                            rd(&p[4 + bj], 0x10) ^ 0x63637,
                            rd(&p[4 + bj], 0x14) ^ 0x63637,
                        );
                        log.push(format!("s12:{},{x},{y},1,1", tok(cur)));
                        log.push(format!("s13:{},0,{},1", tok(cur), a[44]));
                        log.push(format!("s14:{},0,1,1", tok(cur)));
                    }
                    let cnt = i32::from(win(m, p, cur)[0xb5] as i8);
                    for i in 0..cnt.max(0) as usize {
                        wr(&mut p[18 + i], 0xd0, 0);
                    }
                }
                let mut n = [0usize; 3];
                go(a, m, c, p, &mut log, &mut n, 0, a[0], a[1], a[2]);
                if log.is_empty() {
                    log.push("-".to_string());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "go_inside — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "go_inside — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "go_inside — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "go_inside — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Unit::go_inside@0061a2e0: {n} rows agree");
}
