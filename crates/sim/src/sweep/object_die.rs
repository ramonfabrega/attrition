//! `Object::die@00647080` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

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

/// `Object::die@00647080`, run by `tools/emu/sweep_object_die.py` under unicorn
/// (tools/emu): 96 random bytes of object, 3 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c0618c points at 320 random bytes; 4 random bytes at 0x00c0a888; vtable slot +0x150 (3 args) stubbed; the word at `c0+0x12c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 72 random bytes less `0` (placed); the word at `p0+4` points at 72 random bytes less `0` (placed); the word at `p0+8` points at 72 random bytes less `0` (placed); the word at `p0+12` points at 72 random bytes less `0` (placed); the word at `p0+16` points at 72 random bytes less `0` (placed); the word at `p0+20` points at 72 random bytes less `0` (placed); the word at `m+0x18` points at 528 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_die.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Object::die@00647080` after its stubbed `close` (slot 0x150): c0 the objects record (array at +0x12c, count +0x120), p0 the array, p1..p6 its records, p7 the type (`this+0x18`), c1 the word `nuke_effect+0x108`.
            let n = rd(&c[0], 0x120);
            let ammo: Vec<crate::fight::DieAmmo> = (0..6usize)
                .take(usize::try_from(n.max(0)).unwrap_or(0))
                .map(|k| crate::fight::DieAmmo {
                    flags: p[1 + k][4],
                    who: rd(&p[1 + k], 0x3c),
                    idx: rd(&p[1 + k], 0x40),
                    total: rd(&p[1 + k], 0x28),
                    cur: rd(&p[1 + k], 0x24),
                })
                .collect();
            let hold = u16::from_le_bytes([m[0x32], m[0x33]]);
            let sh = i16::from_le_bytes([m[0xa], m[0xb]]);
            let h = crate::fight::die_hold_steps(
                hold,
                m[9],
                sh,
                rd(&p[7], 0x1fc) != 0,
                rd(&c[1], 0),
                &ammo,
            );
            m[0x32..0x34].copy_from_slice(&h.to_le_bytes());
            0
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "die — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "die — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "die — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Object::die@00647080: {n} rows agree");
}
