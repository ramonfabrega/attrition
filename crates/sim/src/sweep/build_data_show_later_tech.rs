//! `BuildData::show_later_tech@0062d920` under the emulator against the port (item 3078; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::show_later_tech@0062d920`, run by `tools/emu/sweep_build_data_show_later_tech.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c061e8 points at 64 random bytes; the global cell 0x00c061e0 points at 16 random bytes; vtable slot +0x4 (0 args) stubbed; callee 0x00668700 (2 args) stubbed; callee 0x006e0c80 (1 args) stubbed; callee 0x00639d10 (2 args) stubbed; the word at `0xc0aabc` points at 48 random bytes less `-4` (placed); the word at `p0+0` points at 464 random bytes less `0` (placed); the word at `p0+4` points at 464 random bytes less `0` (placed); the word at `p0+8` points at 464 random bytes less `0` (placed); the word at `p0+12` points at 464 random bytes less `0` (placed); the word at `p0+16` points at 464 random bytes less `0` (placed); the word at `p0+20` points at 464 random bytes less `0` (placed); the word at `p0+24` points at 464 random bytes less `0` (placed); the word at `p0+28` points at 464 random bytes less `0` (placed); the word at `p0+32` points at 464 random bytes less `0` (placed); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 0, 1] (90%); stub 0 answers 0..3 every row; stub 1 answers -3..7 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_show_later_tech.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the tech, 1 the depth, 2 the prerequisite count (slot +4), 3 what `get_preq` answers
                // (every call), 4 `has_tech`, 5 `queue_here`. The table is `p[1..=8]`, `c[0]` the game (its byte
                // 0x36 the ending technology). Every stub answers the same on every call, so the recursion asks
                // the same questions one depth lower.
                fn go(a: &[i32], p: &[Vec<u8>], c: &[Vec<u8>], tech: i32, depth: i32) -> i32 {
                    let rec = |t: i32| &p[(2 + t) as usize];
                    let rd4 = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
                    if i32::from(c[0][0x36]) < rd4(rec(tech), 0x1c8) {
                        return 0;
                    }
                    for _ in 0..a[2] {
                        let preq = a[3];
                        if preq < -1 {
                            return 0;
                        }
                        if a[4] == 0 {
                            if a[5] != 0 {
                                return 0;
                            }
                            if rd4(rec(preq), 0x40) == 0x1b3 {
                                return 0;
                            }
                            if depth - 1 < 0 {
                                return 0;
                            }
                            if go(a, p, c, preq, depth - 1) == 0 {
                                return 0;
                            }
                        }
                    }
                    1
                }
                i64::from(go(a, p, c, a[0], a[1]))
            }
        };
        assert_eq!(got, want_eax, "show_later_tech — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "show_later_tech — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "show_later_tech — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "show_later_tech — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildData::show_later_tech@0062d920: {n} rows agree");
}
