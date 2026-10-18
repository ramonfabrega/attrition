//! `Unit::find_new_bomber_target@005eb960` under the emulator against the port (item 3234; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_new_bomber_target@005eb960`, run by `tools/emu/sweep_unit_find_new_bomber_target.py` under unicorn
/// (tools/emu): 128 random bytes of object, 5 argument(s), `p` windows in order: p0 argument 4, p1..p20 the place(s), in flag order; the global cell 0x00c0618c points at 576 random bytes; the global cell 0x00c061f0 points at 64 random bytes; argument 4 names 4 random bytes (null a third of the rows); callee 0x0065a120 (10 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x0064e5c0 (4 args) stubbed; the word at `c0+0x214` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed); the word at `p6+12` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 32 random bytes less `0` (placed); the word at `p11+4` points at 32 random bytes less `0` (placed); the word at `p11+8` points at 32 random bytes less `0` (placed); the word at `p11+12` points at 32 random bytes less `0` (placed); the word at `c0+0x4c` points at 16 random bytes less `0` (placed); the word at `p16+0` points at 32 random bytes less `0` (placed); the word at `p16+4` points at 32 random bytes less `0` (placed); the word at `p16+8` points at 32 random bytes less `0` (placed); the word at `p16+12` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 2 answers -4..400 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_new_bomber_target.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: c0 `GameAccess::objects` (count `+0x208`, the hit list `+0x214` -> p1 -> p2..p5, each a
                // record with the type byte `+9` and the signed index short `+10`; the type's position table
                // `+0x14 + 28 * type` -> p6 / p11 / p16, whose records p7..p10 / p12..p15 / p17..p20 hold the
                // xor-0x63637 position at `+0x10` / `+0x14`), c1 `constants` (`+0x30` `bomber_respond_range`),
                // p0 `param_5`. Stubs: 0 `find_builds`, 1 `valid_target` (`a[8..16]` in call order), 2
                // `compare_target` (`a[16..24]`). The stubs leave `ecx` as they were handed it, so the `ecx` the
                // second `find_builds` is handed is the residue of the last `vector_dist` / `valid_target` /
                // quotient before it, which `vd_ecx` and `ecx_tok` follow.
                const K: i32 = 0x63637;
                let range = rd(&c[1], 0x30);
                let (ux, uy) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                let (mut x, mut y) = (a[0], a[1]);
                if rd(m, 0x68) & 0x40000 != 0 {
                    (x, y) = (ux, uy);
                }
                let who = m[9];
                let vd = crate::world::vector_dist;
                // `vector_dist`'s `ecx` on return: twice the long side when it squares, else the `dx` it was handed.
                let vd_ecx = |dx: i32, dy: i32| {
                    let (ax, ay) = (dx.unsigned_abs(), dy.unsigned_abs());
                    let (hi, lo) = if ax > ay { (ax, ay) } else { (ay, ax) };
                    if hi != 0 && lo < 60000 {
                        (2 * hi) as i32
                    } else {
                        dx
                    }
                };
                let mut toks = ["-".to_string(), "-".into(), "-".into()];
                let (dx0, dy0) = (x - ux, y - uy);
                let res: i64 = if vd(dx0, dy0) > range * 0x3c0 {
                    -1
                } else {
                    let n = rd(&c[0], 0x208).max(0) as usize;
                    let mut ecx_tok = vd_ecx(dx0, dy0).to_string();
                    let (mut best, mut ri, mut rt) = (-1i32, -1i32, -1i32);
                    let (mut vn, mut cn) = (0usize, 0usize);
                    let pos = |t: usize, i: usize| {
                        let r = &p[7 + 5 * t + i];
                        (rd(r, 0x10) ^ K, rd(r, 0x14) ^ K)
                    };
                    let mut sep = |t: &mut String, s: String| {
                        if t != "-" {
                            t.push('/');
                            t.push_str(&s);
                        } else {
                            *t = s;
                        }
                    };
                    if a[2] >= 0 {
                        let s = format!(
                            "{ecx_tok},{x},{y},3,{who},{},0,0,0,{ecx_tok},0",
                            range * 0x3c0
                        );
                        sep(&mut toks[0], s);
                        for i in 0..n {
                            let cand = &p[2 + i];
                            let (ty, id) = (
                                cand[9] as usize,
                                i16::from_le_bytes([cand[10], cand[11]]) as i32,
                            );
                            sep(&mut toks[1], format!("m+0x0,{id},{ty},0"));
                            let valid = a[8 + vn];
                            vn += 1;
                            if valid != 0 {
                                sep(&mut toks[2], format!("m+0x0,{id},{ty},1,0"));
                                let cmp = a[16 + cn];
                                cn += 1;
                                let (rx, ry) = pos(ty, id as usize);
                                let d = vd(rx - x, ry - y);
                                let v = cmp / (d / 0xc0 + 1);
                                ecx_tok = v.to_string();
                                if best < v {
                                    (best, ri, rt) = (v, id, ty as i32);
                                }
                            } else {
                                ecx_tok = "m+0x0".to_string();
                            }
                        }
                    }
                    if best < 0 {
                        let s = format!(
                            "{ecx_tok},{x},{y},3,{who},{},0,0,0,{ecx_tok},0",
                            range * 0xc0
                        );
                        sep(&mut toks[0], s);
                        for i in 0..n {
                            let cand = &p[2 + i];
                            let (ty, id) = (
                                cand[9] as usize,
                                i16::from_le_bytes([cand[10], cand[11]]) as i32,
                            );
                            sep(&mut toks[1], format!("m+0x0,{id},{ty},0"));
                            let valid = a[8 + vn];
                            vn += 1;
                            if valid != 0 {
                                let (rx, ry) = pos(ty, id as usize);
                                let d = vd(rx - x, ry - y);
                                if d <= range * 0xc0 {
                                    sep(&mut toks[2], format!("m+0x0,{id},{ty},1,0"));
                                    let cmp = a[16 + cn];
                                    cn += 1;
                                    let v = cmp / (d / 0xc0 + 1);
                                    if best < v {
                                        (best, ri, rt) = (v, id, ty as i32);
                                    }
                                }
                            }
                        }
                    }
                    if a[4] != 0 {
                        wr(&mut p[0], 0, if best >= 0 { rt } else { -1 });
                    }
                    if best >= 0 {
                        i64::from(ri)
                    } else {
                        -1
                    }
                };
                log.extend(toks);
                res
            }
        };
        assert_eq!(got, want_eax, "find_new_bomber_target — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_new_bomber_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_new_bomber_target — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_new_bomber_target — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "find_new_bomber_target — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::find_new_bomber_target@005eb960: {n} rows agree");
}
