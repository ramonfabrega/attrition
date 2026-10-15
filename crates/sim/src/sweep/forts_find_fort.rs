//! `Forts::find_fort@0073ebe0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `Forts::find_fort@0073ebe0`, run by `tools/emu/sweep_forts_find_fort.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 16 random bytes less `0` (placed); the word at `p4+12` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed); the word at `p5+12` points at 16 random bytes less `0` (placed); the word at `0xc0618c` points at 64 random bytes less `0` (placed); the word at `p14+0x14` points at 16 random bytes less `0` (placed); the word at `p14+0x30` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 24 random bytes less `0` (placed); the word at `p15+4` points at 24 random bytes less `0` (placed); the word at `p15+8` points at 24 random bytes less `0` (placed); the word at `p15+12` points at 24 random bytes less `0` (placed); the word at `p16+0` points at 24 random bytes less `0` (placed); the word at `p16+4` points at 24 random bytes less `0` (placed); the word at `p16+8` points at 24 random bytes less `0` (placed); the word at `p16+12` points at 24 random bytes less `0` (placed); the word at `0xcae5fc` points at 64 random bytes less `0` (placed); 2 rows 0x6eec apart at `@0xe3a7b8`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc0a490`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_forts_find_fort.py") else {
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
            // The first built fort of the player's list (`who` = the third argument, the count at the image's
            // `0xe3a7b8 + 0x6eec * who`) whose object's two handles (xor `0x63637`, `>> 8`, through the handle
            // table at `0xcae5fc`) are the first two arguments; else -1.
            let who = a[2] as usize;
            let count = rd(&p[who], 0);
            let handle = |obj: &[u8], at: usize| rd(&p[25], 4 * ((rd(obj, at) ^ 0x63637) >> 8) as usize);
            (0..count as usize)
                .find(|&k| {
                    let fort = &p[6 + 4 * who + k];
                    if fort[6] & 1 == 0 {
                        return false;
                    }
                    let idx = i16::from_le_bytes([fort[2], fort[3]]) as usize;
                    let obj = &p[17 + 4 * who + idx];
                    handle(obj, 0x10) == a[0] && handle(obj, 0x14) == a[1]
                })
                .map_or(-1, |k| k as i64)
        };
        assert_eq!(got, want_eax, "find_fort — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_fort — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_fort — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_fort — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Forts::find_fort@0073ebe0: {n} rows agree");
}
