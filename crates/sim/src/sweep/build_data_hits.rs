//! `BuildData::hits@0062e740` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::hits@0062e740`, run by `tools/emu/sweep_build_data_hits.py` under unicorn
/// (tools/emu): 220 random bytes of object, 1 argument(s), vtable slot +0x4c (0 args) stubbed; callee 0x006508c0 (1 args) stubbed; the word at `m+0x8c` points at 8 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 1 answered from [0, 1, 7, 50, 100, 1000, -5] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_hits.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The hit points `m+0x54` (`a[0] == 0`) or `m+0x20`, scaled (float, `cvttss2si`: out of range
            // and NaN are `i32::MIN`) by the build queue's progress when the building is active (slot +0x4c,
            // `a[1]`, or bit 4 of `m[8]` when laid as the base, flag `a[3]`), `queued` (`m[0x82]`), a
            // non-empty queue (`m+0x88`) and its head type (the short at `p0+4`) 0x29a or 0x286: the
            // queue's time `rd(p[0], 0)` against `train_time` (`a[2]`), at least 1 (item 2844).
            let h = if a[0] == 0 { rd(m, 0x54) } else { rd(m, 0x20) };
            let active = if a[3] != 0 { m[8] & 4 != 0 } else { a[1] != 0 };
            let ty = i32::from(i16::from_le_bytes([p[0][4], p[0][5]]));
            if active && m[0x82] != 0 && rd(m, 0x88) > 0 && (ty == 0x29a || ty == 0x286) {
                let t = a[2];
                let q = rd(&p[0], 0);
                let f = (t.wrapping_sub(q) as f32 * h as f32) / t as f32;
                let r = if f.is_nan() || f >= 2147483648.0 || f < -2147483648.0 { i32::MIN } else { f as i32 };
                i64::from(if r < 1 { 1 } else { r })
            } else {
                i64::from(h)
            }
        };
        assert_eq!(got, want_eax, "hits — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "hits — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "hits — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "hits — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::hits@0062e740: {n} rows agree");
}
