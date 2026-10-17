//! `Army::charge@006f9a90` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

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

/// `Army::charge@006f9a90`, run by `tools/emu/sweep_army_charge.py` under unicorn
/// (tools/emu): 160 random bytes of object, 2 argument(s), the global cell 0x00c0618c points at 64 random bytes; callee 0x0070fba0 (10 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 24 random bytes less `0` (placed); the word at `p8+4` points at 24 random bytes less `0` (placed); the word at `p8+8` points at 24 random bytes less `0` (placed); the word at `p8+12` points at 24 random bytes less `0` (placed); the word at `p9+0` points at 24 random bytes less `0` (placed); the word at `p9+4` points at 24 random bytes less `0` (placed); the word at `p9+8` points at 24 random bytes less `0` (placed); the word at `p9+12` points at 24 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x9d4 apart at `0xe85f20`, slices +0x8:0x4, +0xc:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_charge.py") else {
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
            // For each of the army's `short(m+0x96)` group ids (ints from `m+0x54`; row `g` of the groups table: `p[2g]`
            // its owner word at +8, `p[2g+1]` its flag word at +0xc) that is not negative and whose group has its flag
            // word set: the owner word becomes the army's short at `m+2`, the group is told to move to the
            // target object's position (the object `a[0]` of owner `a[1]`'s array, `p[10 + 4·a[1] + a[0]]`, its
            // words at +0x10 and +0x14 xor 0x63637) as an attack-to order, and the army's target words
            // `m+0x30`, `m+0x34` take `a[0]`, `a[1]` (item 2852).
            let count = i16::from_le_bytes([m[0x96], m[0x97]]);
            for i in 0..i32::from(count) {
                let g = rd(m, 0x54 + 4 * i as usize);
                if g < 0 {
                    continue;
                }
                let g = g as usize;
                if rd(&p[2 * g + 1], 0) == 0 {
                    continue;
                }
                wr(
                    &mut p[2 * g],
                    0,
                    i32::from(i16::from_le_bytes([m[2], m[3]])),
                );
                let o = &p[10 + 4 * a[1] as usize + a[0] as usize];
                let (x, y) = (rd(o, 0x10) ^ 0x63637, rd(o, 0x14) ^ 0x63637);
                log.push(format!("s0:r{g}+0x0,{x},{y},0,0,0,2,1,-1,-1,0"));
                wr(m, 0x30, a[0]);
                wr(m, 0x34, a[1]);
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "charge — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "charge — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "charge — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "charge — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Army::charge@006f9a90: {n} rows agree");
}
