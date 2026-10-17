//! `GroupData::has_air_units@0070b250` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::has_air_units@0070b250`, run by `tools/emu/sweep_group_data_has_air_units.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 0 argument(s), callee 0x00646c40 (1 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 68 random bytes less `0` (placed); the word at `p0+4` points at 68 random bytes less `0` (placed); the word at `p0+8` points at 68 random bytes less `0` (placed); the word at `p0+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 68 random bytes less `0` (placed); the word at `p5+4` points at 68 random bytes less `0` (placed); the word at `p5+8` points at 68 random bytes less `0` (placed); the word at `p5+12` points at 68 random bytes less `0` (placed); the word at `p1+0x18` points at 544 random bytes less `0` (placed); the word at `p2+0x18` points at 544 random bytes less `0` (placed); the word at `p3+0x18` points at 544 random bytes less `0` (placed); the word at `p4+0x18` points at 544 random bytes less `0` (placed); the word at `p6+0x18` points at 544 random bytes less `0` (placed); the word at `p7+0x18` points at 544 random bytes less `0` (placed); the word at `p8+0x18` points at 544 random bytes less `0` (placed); the word at `p9+0x18` points at 544 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; figures p1..p4 link at +0x28 (2 bytes) above their own index or to -1; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_has_air_units.py") else {
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
            {
                // The list names an object through `objects[who]` (ptrtab: table w's records are p[1 + 5w + i]); a
                // can_carry(AIR) answer (stub 0, a[1..]) opens its passenger chain (+0x28 index, +0x3e table, both
                // -1 to end), and a passenger whose type record (+0x18, its +0x218 word) is 2 answers 1.
                let num = rd(m, 12);
                let who = usize::from(m[0x4a]);
                let ty = |w: usize, i: usize| &p[10 + 4 * w + i];
                let mut calls = 0usize;
                let mut hit = 0i64;
                'list: for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i = usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]])).expect("an index");
                    let r = a[1 + calls];
                    calls += 1;
                    if r != 0 {
                        let o = &p[1 + 5 * who + i];
                        let mut nx = i16::from_le_bytes([o[0x28], o[0x29]]);
                        let mut nw = o[0x3e] as i8;
                        while nx >= 0 && nw >= 0 {
                            let (w2, i2) = (usize::try_from(nw).expect("a table"), usize::try_from(nx).expect("an index"));
                            if rd(ty(w2, i2), 0x218) == 2 {
                                hit = 1;
                                break 'list;
                            }
                            let o2 = &p[1 + 5 * w2 + i2];
                            nx = i16::from_le_bytes([o2[0x28], o2[0x29]]);
                            nw = o2[0x3e] as i8;
                        }
                    }
                }
                hit
            }

        };
        assert_eq!(got, want_eax, "has_air_units — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_air_units — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_air_units — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_air_units — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::has_air_units@0070b250: {n} rows agree");
}
