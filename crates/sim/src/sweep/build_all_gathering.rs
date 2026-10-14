//! `Build::all_gathering@0062f570` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

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

/// `Build::all_gathering@0062f570`, run by `tools/emu/sweep_build_all_gathering.py` under unicorn
/// (tools/emu): 116 random bytes of object, 0 argument(s), callee 0x0062f710 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; the word at `0xc0aec0+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p0+4` points at 224 random bytes less `0` (placed); the word at `p0+8` points at 224 random bytes less `0` (placed); the word at `p0+12` points at 224 random bytes less `0` (placed); the word at `p0+16` points at 224 random bytes less `0` (placed); the word at `p0+20` points at 224 random bytes less `0` (placed); the word at `p1+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 4 random bytes less `0` (placed, null a third of the rows); the word at `p2+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p11+8` points at 4 random bytes less `0` (placed, null a third of the rows); the word at `p3+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p13+4` points at 16 random bytes less `0` (placed); the word at `p14+8` points at 4 random bytes less `0` (placed, null a third of the rows); the word at `p4+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p16+4` points at 16 random bytes less `0` (placed); the word at `p17+8` points at 4 random bytes less `0` (placed, null a third of the rows); the word at `p5+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p19+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 4 random bytes less `0` (placed, null a third of the rows); the word at `p6+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p22+4` points at 16 random bytes less `0` (placed); the word at `p23+8` points at 4 random bytes less `0` (placed, null a third of the rows); stub 1 answers the address of 40 random bytes, a `p` window after the places; figures p1..p6 link at +0x92 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_all_gathering.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The chain head `+0x70`, the player `+9` (held at 2), the
                // units table `p[0]` of six units `p[1..7]`, each's next
                // `+0x92`; unit k's order slot `+0xdc` is `p[7 + 3k]`, whose
                // `+4` is `p[8 + 3k]`, whose `+8` is the order `p[9 + 3k]`
                // (the stubbed vtable; either pointer null a third of the
                // rows); the walk caches `+0xd4`, `+0xcc` and `+0xd0`. Every
                // order's `+0xe4` answers the record `p[25]`: `+0x24`
                // `goto_build`, `+0x20` `wait`. `check_gatherers` is a stub.
                let mut chain = Vec::new();
                let mut at = i16::from_le_bytes([m[0x70], m[0x71]]);
                while at >= 0 {
                    chain.push(at as usize);
                    at = i16::from_le_bytes([p[1 + at as usize][0x92], p[1 + at as usize][0x93]]);
                }
                let rec = (p[25][0x24] != 0, rd(&p[25], 0x20));
                let ok = crate::orders::all_at_work(chain, |u| {
                    if rd(&p[1 + u], 0xdc) == 0 {
                        return None;
                    }
                    let (d, r) = (p[7 + 3 * u][4..8].to_vec(), p[8 + 3 * u].clone());
                    p[1 + u][0xd4..0xd8].copy_from_slice(&d);
                    p[1 + u][0xcc..0xd0].copy_from_slice(&r[8..12]);
                    p[1 + u][0xd0] = r[0xc];
                    (rd(&r, 8) != 0).then_some(rec)
                });
                i64::from(ok)
            }
        };
        assert_eq!(got, want_eax, "all_gathering — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "all_gathering — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "all_gathering — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "all_gathering — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::all_gathering@0062f570: {n} rows agree");
}
