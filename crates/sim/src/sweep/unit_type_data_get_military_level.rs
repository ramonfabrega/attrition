//! `UnitTypeData::get_military_level@0061d4c0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `UnitTypeData::get_military_level@0061d4c0`, run by `tools/emu/sweep_unit_type_data_get_military_level.py` under unicorn
/// (tools/emu): 768 random bytes of object, 0 argument(s), vtable slot +0x38 (0 args) stubbed; callee 0x00668700 (2 args) stubbed; the word at `0xe85ddc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); the word at `0xc0aabc` points at 32 random bytes less `0` (placed); the word at `p9+0` points at 32 random bytes less `0` (placed); the word at `p9+4` points at 32 random bytes less `0` (placed); the word at `p9+8` points at 32 random bytes less `0` (placed); the word at `p9+12` points at 32 random bytes less `0` (placed); the word at `p9+16` points at 32 random bytes less `0` (placed); the word at `p9+20` points at 32 random bytes less `0` (placed); the word at `p9+24` points at 32 random bytes less `0` (placed); the word at `p9+28` points at 32 random bytes less `0` (placed); slot +0x38 laid as 0x00470870 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 answers -1..7 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_get_military_level.py") else {
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
            'g: {
    // get_military_level: a stored level (>= 0) is returned; else the type's own tech, if it is an epoch tech (index
    // 0x227..0x242, or the slot-0x38 answer) with nothing in the tech record's +0x14, else its first prerequisite
    // (get_preq(1, -1)) under the same test, answering the index less 0x23b -- or 0.
    let _ = (&c, &a[0]);
    let level = rd(m, 0x2dc);
    if level >= 0 {
        break 'g level as i64;
    }
    let base = a[4] != 0;
    let mut calls = 0;
    let mut epoch_and_free = |t: i32, p: &[Vec<u8>]| -> bool {
        let slot = a[2 + calls.min(1)];
        calls += 1;
        let epoch = if base {
            let idx = rd(&p[1 + t as usize], 4);
            (0x227..0x243).contains(&idx)
        } else {
            slot != 0
        };
        epoch && rd(&p[10 + t as usize], 0x14) == 0
    };
    let mut t = rd(m, 0x30);
    if !(t >= 0 && epoch_and_free(t, p)) {
        t = a[1];
    }
    if t >= 0 && epoch_and_free(t, p) {
        (t - 0x23b) as i64
    } else {
        0
    }
}
        };
        assert_eq!(got, want_eax, "get_military_level — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_military_level — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_military_level — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_military_level — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::get_military_level@0061d4c0: {n} rows agree");
}
