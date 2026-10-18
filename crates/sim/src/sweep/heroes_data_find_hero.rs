//! `HeroesData::find_hero@0073a1b0` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

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

/// `HeroesData::find_hero@0073a1b0`, run by `tools/emu/sweep_heroes_data_find_hero.py` under unicorn
/// (tools/emu): 16 random bytes of object, 6 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p20 the place(s), in flag order; the global cell 0x00c061e4 points at 3168 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00739e50 (0 args) stubbed; callee 0x0046cff0 (0 args) stubbed; the word at `0xc0a130+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 48 random bytes less `0` (placed); the word at `p2+4` points at 48 random bytes less `0` (placed); the word at `p2+8` points at 48 random bytes less `0` (placed); the word at `p2+12` points at 48 random bytes less `0` (placed); the word at `0xc0aec0+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 144 random bytes less `0` (placed); the word at `p7+4` points at 144 random bytes less `0` (placed); the word at `p7+8` points at 144 random bytes less `0` (placed); the word at `p7+12` points at 144 random bytes less `0` (placed); the word at `0xc0ab84+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `p12+12` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 8 random bytes less `0` (placed); the word at `p9+0x18` points at 8 random bytes less `0` (placed); the word at `p10+0x18` points at 8 random bytes less `0` (placed); the word at `p11+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [1, 1, 0] (90%); stub 1 answered from [1, 1, 0] (90%); stub 2 answered from [1, 1, 0] (90%); stub 3 answered from [1, 1, 0] (90%); stub 4 answered from [0, 1, 2, 3] (90%); stub 5 answered from [0, 1, 2, 191, 192, 193, 383, 384, 385, 575, 576, 577] (90%); stub 5 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `0xc061e0`, slices +0x420:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_heroes_data_find_hero.py") else {
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
            // `a[0..6]` the stack arguments (two coordinates, the player, the mask, the type, the
            // margin), `a[6..12]` the answers of slots 0x8, 0xbc, 0xb8, 0x60 and the direct callees
            // `get_radius` and `vector_dist`, `a[12]` the flag that lays `is` inline for slot 0xb8.
            // p[0], p[1] the leaders' counts (`+0x420`), p[3 + i] the hero records, p[8 + k] the
            // units, p[13 + k] their coordinates (words xor 0x63637).
            let player = a[2] as usize;
            let count = rd(&p[player], 0);
            let mut calls: Vec<String> = Vec::new();
            let mut found = -1i64;
            for i in 0..count as usize {
                let hero = &p[3 + i];
                if hero[0x28] & 1 == 0 {
                    continue;
                }
                let k = i16::from_le_bytes([hero[0x26], hero[0x27]]) as usize;
                if a[6] == 0 || a[7] == 0 {
                    continue;
                }
                if a[4] >= 0 && (if a[12] != 0 { a[9] } else { a[8] }) == 0 {
                    continue;
                }
                let at = &p[13 + k];
                let dx = a[0].wrapping_sub(rd(at, 0x10) ^ 0x63637).wrapping_abs();
                let dy = a[1].wrapping_sub(rd(at, 0x14) ^ 0x63637).wrapping_abs();
                calls.push(format!("{dx},{dy}"));
                if a[11].wrapping_sub(a[5]) > a[10].wrapping_mul(0xc0) {
                    continue;
                }
                if a[3] == 0 || rd(&p[8 + k], 0x68) & a[3] != 0 {
                    found = k as i64;
                    break;
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            found
        };
        assert_eq!(got, want_eax, "find_hero — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_hero — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_hero — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_hero — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "find_hero — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("HeroesData::find_hero@0073a1b0: {n} rows agree");
}
