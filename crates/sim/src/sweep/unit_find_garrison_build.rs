//! `Unit::find_garrison_build@00605040` under the emulator against the port (item 3229; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_garrison_build@00605040`, run by `tools/emu/sweep_unit_find_garrison_build.py` under unicorn
/// (tools/emu): 128 random bytes of object, 2 argument(s), `p` windows in order: p0..p30 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x178 (0 args) stubbed; callee 0x00633f50 (1 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x00609d40 (1 args) stubbed; callee 0x0061d7d0 (1 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `m+0x60` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 128 random bytes less `0` (placed); the word at `p7+4` points at 128 random bytes less `0` (placed); the word at `p7+8` points at 128 random bytes less `0` (placed); the word at `p7+12` points at 128 random bytes less `0` (placed); the word at `p7+16` points at 128 random bytes less `0` (placed); the word at `p7+20` points at 128 random bytes less `0` (placed); the word at `0xc09970` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 16 random bytes less `0` (placed); the word at `p14+4` points at 16 random bytes less `0` (placed); the word at `p14+8` points at 16 random bytes less `0` (placed); the word at `p14+12` points at 16 random bytes less `0` (placed); the word at `0xc0998c` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 16 random bytes less `0` (placed); the word at `p19+4` points at 16 random bytes less `0` (placed); the word at `p19+8` points at 16 random bytes less `0` (placed); the word at `p19+12` points at 16 random bytes less `0` (placed); the word at `p8+0x18` points at 8 random bytes less `0` (placed); the word at `p9+0x18` points at 8 random bytes less `0` (placed); the word at `p10+0x18` points at 8 random bytes less `0` (placed); the word at `p11+0x18` points at 8 random bytes less `0` (placed); the word at `p12+0x18` points at 8 random bytes less `0` (placed); the word at `p13+0x18` points at 8 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `p8+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p9+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p10+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p11+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p12+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p13+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `c0+0x30` laid as the address of `p0` every row; stub 0 answers `p8+j`, the data of the object its `ecx` is in `p1+j` (j < 6), linked at `+0x74`; stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 4 answers 6 values in call order, printed after the answers; stub 5 answers 6 values in call order, printed after the answers; stub 6 answers 6 values in call order, printed after the answers; stub 1 answers 0..3 every row; stub 2 answers -1..6 every row; stub 3 answers 0..6 every row; stub 4 answers 0..2 every row; stub 5 answers 0..1 every row; stub 6 answers 0..3 every row; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_garrison_build.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The original's `vector_dist@0046cff0` on its two register arguments (`ecx`, `edx`): the sign-trick
                // absolute values, the larger of the two over the smaller as `hi + lo*lo/(2*hi)` (unsigned, the
                // smaller under 60000) or `(hi + 2*lo) >> 1` — written with the machine's wraps, not the crate's.
                let vd = |ecx: i32, edx: i32| -> i32 {
                    let (x, y) = (ecx.wrapping_abs(), edx.wrapping_abs());
                    let (hi, lo) = if x > y {
                        (x as u32, y as u32)
                    } else {
                        (y as u32, x as u32)
                    };
                    if hi == 0 {
                        return 0;
                    }
                    if lo >= 60000 {
                        (lo.wrapping_add(hi.wrapping_mul(2)) >> 1) as i32
                    } else {
                        (lo.wrapping_mul(lo) / hi.wrapping_mul(2)).wrapping_add(hi) as i32
                    }
                };
                // The coordinates are the OBJECT record's (+0x10, +0x14), not the data's.
                // The answers after the two arguments: seven one-a-stub words (unused here), then six a stub, six
                // calls deep and the last repeated: slot +0x4c at a[9..], slot +0x178 at a[15..], the limit at
                // a[21..], `num_inside` at a[27..], `control_cost` at a[33..], `can_garrison` at a[39..].
                let mut turn = [0usize; 6];
                let mut ask = |s: usize| -> i32 {
                    let v = a[9 + 6 * s + turn[s].min(5)];
                    turn[s] += 1;
                    v
                };
                let (mut l3, mut l4, mut l5, mut l6): (
                    Vec<String>,
                    Vec<String>,
                    Vec<String>,
                    Vec<String>,
                ) = (vec![], vec![], vec![], vec![]);
                let s16 = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                let mut best: i32 = -1;
                let mut best_d: i32 = 99999999;
                if a[0] >= 0 {
                    // The city table of the player (`0xc09970 + 0x1c * who`): its record at +4 bit 0 and the first
                    // object at the short +8.
                    let city = &p[(if a[1] == 0 { 15 } else { 20 }) + a[0] as usize];
                    if city[4] & 1 != 0 {
                        let mut o = i32::from(s16(city, 8));
                        while o >= 0 {
                            let k = o as usize;
                            let link = i32::from(s16(&p[8 + k], 0x74));
                            if p[8 + k][8] & 1 != 0 {
                                let is_build = rd(&p[8 + k], 0) as u32 == 0xb42174;
                                let ok = if is_build {
                                    p[8 + k][8] & 4 != 0
                                } else {
                                    ask(0) != 0
                                };
                                if ok {
                                    let lim = if is_build {
                                        l3.push(format!("p{}+0x0,{}", 24 + k, p[8 + k][9]));
                                        ask(2)
                                    } else {
                                        ask(1)
                                    };
                                    l4.push(format!("p{}+0x0,0", 8 + k));
                                    let num = ask(3);
                                    l5.push("m+0x0,0".to_string());
                                    let cost = ask(4);
                                    if num.wrapping_add(cost) <= lim {
                                        l6.push(format!("p30+0x0,{}", rd(&p[24 + k], 4)));
                                        if ask(5) != 0 {
                                            let dx = (rd(&p[1 + k], 0x10) ^ 0x63637)
                                                .wrapping_sub(rd(m, 0x10) ^ 0x63637);
                                            let dy = (rd(&p[1 + k], 0x14) ^ 0x63637)
                                                .wrapping_sub(rd(m, 0x14) ^ 0x63637);
                                            let d = vd(dx, dy);
                                            if d < best_d {
                                                best = o;
                                                best_d = d;
                                            }
                                        }
                                    }
                                }
                            }
                            o = link;
                        }
                    }
                }
                let j = |l: Vec<String>| {
                    if l.is_empty() {
                        "-".to_string()
                    } else {
                        l.join("/")
                    }
                };
                log = vec![j(l3), j(l4), j(l5), j(l6)];
                i64::from(best)
            }
        };
        assert_eq!(got, want_eax, "find_garrison_build — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_garrison_build — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_garrison_build — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_garrison_build — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "find_garrison_build — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::find_garrison_build@00605040: {n} rows agree");
}
