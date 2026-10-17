//! `Groups::copy_group@006fa690` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `Groups::copy_group@006fa690`, run by `tools/emu/sweep_groups_copy_group.py` under unicorn
/// (tools/emu): 2560 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c061ec points at 1536 random bytes; callee 0x0055e0ac (3 args) stubbed; the word at `m+0x30` points at 2560 random bytes less `0` (placed); stub 0 is `__cdecl`: the caller pops its arguments; stub 0 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); `edx` the address of window `p0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_groups_copy_group.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `this` (ecx) takes the group in `edx`: the bytes at +0x4a and +0x49, the words at +0xc, +0x18..+0x24
            // and +0x40, the frame stamp (+0x14, from the game cell's +0x550), then, when the count at +0xc is
            // non-zero, six parallel arrays (+0x8cc by 2 bytes, +0x84c by 1, +0x4c/+0x24c/+0x44c/+0x64c by 4).
            let src = &p[0];
            for off in [0x4a, 0x49] {
                m[off] = src[off];
            }
            for off in [0xc, 0x18, 0x1c, 0x20, 0x24, 0x40] {
                m[off..off + 4].copy_from_slice(&src[off..off + 4]);
            }
            wr(m, 0x14, rd(&c[0], 0x550));
            let n = rd(src, 0xc) as usize;
            for (off, w) in [
                (0x8cc, 2),
                (0x84c, 1),
                (0x4c, 4),
                (0x24c, 4),
                (0x44c, 4),
                (0x64c, 4),
            ] {
                if n != 0 {
                    m[off..off + n * w].copy_from_slice(&src[off..off + n * w]);
                }
            }
            // The answer is the last copy's destination (this + 0x64c; `this` is the object's base 0x20000000),
            // else the zero count last loaded.
            if n != 0 { 0x2000_064c } else { 0 }
        };
        assert_eq!(got, want_eax, "copy_group — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "copy_group — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "copy_group — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "copy_group — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Groups::copy_group@006fa690: {n} rows agree");
}
