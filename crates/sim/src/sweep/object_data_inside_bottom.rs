//! `ObjectData::inside_bottom@00644000` under the emulator against the port (item 2709; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::inside_bottom@00644000`, run by `tools/emu/sweep_object_data_inside_bottom.py` under unicorn
/// (tools/emu): 12 random bytes of object, 1 argument(s), argument 0 names 4 random bytes (null a third of the rows); the word at `0xc0ab84+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 68 random bytes less `0` (placed); the word at `p1+4` points at 68 random bytes less `0` (placed); the word at `p1+8` points at 68 random bytes less `0` (placed); the word at `p1+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 68 random bytes less `0` (placed); the word at `p6+4` points at 68 random bytes less `0` (placed); the word at `p6+8` points at 68 random bytes less `0` (placed); the word at `p6+12` points at 68 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 68 random bytes less `0` (placed); the word at `p11+4` points at 68 random bytes less `0` (placed); the word at `p11+8` points at 68 random bytes less `0` (placed); the word at `p11+12` points at 68 random bytes less `0` (placed); figures p2..p5 link at +0x28 (2 bytes) below their own index or to -1, -2; figures p7..p10 link at +0x28 (2 bytes) below their own index or to -1, -2; figures p12..p15 link at +0x28 (2 bytes) below their own index or to -1, -2; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_inside_bottom.py") else {
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
            let _ = &c;
let r16 = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
let rec = |who: usize, idx: usize| 2 + 5 * who + idx;
let mut who = usize::from(m[9]);
let mut idx = usize::try_from(r16(m, 0xa)).expect("a column");
let mut up = r16(&p[rec(who, idx)], 0x28);
while up >= 0 {
    who = usize::from(p[rec(who, idx)][0x3e]);
    idx = usize::try_from(up).expect("a column");
    up = r16(&p[rec(who, idx)], 0x28);
}
if a[0] != 0 {
    wr(&mut p[0], 0, i32::try_from(who).expect("a row"));
}
i64::try_from(idx).expect("a column")
        };
        assert_eq!(got, want_eax, "inside_bottom — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "inside_bottom — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "inside_bottom — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "inside_bottom — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::inside_bottom@00644000: {n} rows agree");
}
