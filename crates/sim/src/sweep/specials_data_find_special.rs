//! `SpecialsData::find_special@00740070` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `SpecialsData::find_special@00740070`, run by `tools/emu/sweep_specials_data_find_special.py` under unicorn
/// (tools/emu): 64 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p20 the place(s), in flag order; the global cell 0x00c061e4 points at 3168 random bytes; vtable slot +0xbc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0046cff0 (0 args) stubbed; the word at `0xc0a290+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 48 random bytes less `0` (placed); the word at `p2+4` points at 48 random bytes less `0` (placed); the word at `p2+8` points at 48 random bytes less `0` (placed); the word at `p2+12` points at 48 random bytes less `0` (placed); the word at `0xc0aec0+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 144 random bytes less `0` (placed); the word at `p7+4` points at 144 random bytes less `0` (placed); the word at `p7+8` points at 144 random bytes less `0` (placed); the word at `p7+12` points at 144 random bytes less `0` (placed); the word at `0xc0ab84+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `p12+12` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 8 random bytes less `0` (placed); the word at `p9+0x18` points at 8 random bytes less `0` (placed); the word at `p10+0x18` points at 8 random bytes less `0` (placed); the word at `p11+0x18` points at 8 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 383, 384, 385, 767, 768, 769, 1151, 1152, 1153] (90%); stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `0xc061e0`, slices +0x43c:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_specials_data_find_special.py") else {
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
            // `a` is the point (x, y), the player, an unused fourth, then the answers: slots
            // `+0xbc` (`is_on_map`), `+0xb8` (`is`), the type's `+0x60` and `vector_dist`, and
            // the two flags that lay the image's own `is_on_map` and `ObjectData::is` in
            // place of their stubs. The player's specials count is `p0`/`p1`; its records
            // `p3..p6` (flag bit 0 at `+0x28`, the unit's index the short at `+0x26`) name
            // units `p8..p11` and objects `p13..p16`, whose coordinates (`+0x10`, `+0x14`,
            // keyed 0x63637) are measured from the point. A special is found when its
            // unit is on the map, is within `general_radius * 0x180` and is a general.
            let count = rd(&p[a[2] as usize], 0);
            let radius = rd(&c[0], 0xc58).wrapping_mul(0x180);
            let mut calls: Vec<String> = Vec::new();
            let mut found = -1i64;
            for i in 0..count.max(0) as usize {
                let rec = &p[3 + i];
                if rec[0x28] & 1 == 0 {
                    continue;
                }
                let idx = i16::from_le_bytes([rec[0x26], rec[0x27]]) as usize;
                let unit = &p[8 + idx];
                let on_map = if a[8] != 0 {
                    i32::from(u16::from_le_bytes([unit[0x82], unit[0x83]]) >> 15)
                } else {
                    a[4]
                };
                if on_map == 0 {
                    continue;
                }
                let obj = &p[13 + idx];
                let dx = a[0].wrapping_sub(rd(obj, 0x10) ^ 0x63637).wrapping_abs();
                let dy = a[1].wrapping_sub(rd(obj, 0x14) ^ 0x63637).wrapping_abs();
                calls.push(format!("{dx},{dy}"));
                if a[7] > radius {
                    continue;
                }
                let is = if a[9] != 0 { a[6] } else { a[5] };
                if is != 0 {
                    found = i as i64;
                    break;
                }
            }
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            found
        };
        assert_eq!(got, want_eax, "find_special — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_special — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_special — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_special — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "find_special — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("SpecialsData::find_special@00740070: {n} rows agree");
}
