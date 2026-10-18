//! `ObjectsData::find_good_at@0065bec0` under the emulator against the port (item 3134; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_good_at@0065bec0`, run by `tools/emu/sweep_objects_data_find_good_at_wcoord.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), `p` windows in order: p0..p29 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c0a0f0 points at 64 random bytes; 4 random bytes at 0x00c0acc4 (each word in 0..4); callee 0x006e33a0 (2 args) stubbed; the word at `0xcae5fc` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 512 random bytes less `0` (placed); the word at `c1+0` points at 32 random bytes less `0` (placed); the word at `c1+4` points at 32 random bytes less `0` (placed); the word at `c1+8` points at 32 random bytes less `0` (placed); the word at `c1+12` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `0xc0abd8` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 48 random bytes less `0` (placed); the word at `p10+4` points at 48 random bytes less `0` (placed); the word at `p10+8` points at 48 random bytes less `0` (placed); the word at `p10+12` points at 48 random bytes less `0` (placed); the word at `p11+0` points at 48 random bytes less `0` (placed); the word at `p11+4` points at 48 random bytes less `0` (placed); the word at `p11+8` points at 48 random bytes less `0` (placed); the word at `p11+12` points at 48 random bytes less `0` (placed); the word at `p12+0` points at 48 random bytes less `0` (placed); the word at `p12+4` points at 48 random bytes less `0` (placed); the word at `p12+8` points at 48 random bytes less `0` (placed); the word at `p12+12` points at 48 random bytes less `0` (placed); the word at `p13+0` points at 48 random bytes less `0` (placed); the word at `p13+4` points at 48 random bytes less `0` (placed); the word at `p13+8` points at 48 random bytes less `0` (placed); the word at `p13+12` points at 48 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_good_at_wcoord.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let rd = |w: &[u8], o: usize| {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                };
                let rd16 = |w: &[u8], o: usize| {
                    i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
                };
                const KEY: i32 = 0x63637;
                let (x, y, who, want_idx, by_good) = (a[0], a[1], a[2], a[3], a[4]);
                // a[5] the type_avail stub's answer; windows: c0 the world, c2 the good count, p0 the div_3_table,
                // p1 the world's cells, p2.. the goods, p6.. a good's type record, p14+4*pl+j the object arrays.
                let avail = |ty: i32, log: &mut Vec<String>| {
                    log.push(format!(
                        "{},{},1",
                        0x00e3_a390 + who.wrapping_mul(0x6eec),
                        ty
                    ));
                    a[5] != 0
                };
                let ty = |g: usize| rd(&p[6 + g], 4);
                if by_good == 0 {
                    let w = rd(&c[0], 0);
                    let at = 28 * y.wrapping_mul(w).wrapping_add(x) as usize + 8;
                    let (mut idx, mut pl) = (rd16(&p[1], at), rd16(&p[1], at + 2));
                    while idx >= 0 {
                        let o = &p[14 + 4 * pl as usize + idx as usize];
                        idx = rd16(o, 0x2c);
                        pl = rd16(o, 0x2e);
                    }
                    if idx == -2 && rd(&p[2 + pl as usize], 8) & 1 != 0 && ty(pl as usize) != 5 {
                        if want_idx != 0 {
                            i64::from(pl)
                        } else if who < 0 || avail(ty(pl as usize), &mut log) {
                            i64::from(ty(pl as usize))
                        } else {
                            -1
                        }
                    } else {
                        -1
                    }
                } else {
                    let tab = |v: i32| rd(&p[0], 4 * ((v ^ KEY) >> 8) as usize);
                    let mut out = -1i64;
                    for i in 0..rd(&c[2], 0).max(0) as usize {
                        let g = &p[2 + i];
                        if rd(g, 8) & 1 != 0 && tab(rd(g, 0x10)) == x && tab(rd(g, 0x14)) == y {
                            out = if want_idx != 0 {
                                i as i64
                            } else if who < 0 || avail(ty(i), &mut log) {
                                i64::from(ty(i))
                            } else {
                                -1
                            };
                            break;
                        }
                    }
                    out
                }
            }
        };
        if log.is_empty() {
            log.push("-".to_string());
        }
        assert_eq!(got, want_eax, "find_good_at — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_good_at — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_good_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_good_at — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_good_at — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("ObjectsData::find_good_at@0065bec0: {n} rows agree");
}
