//! `ObjectTypeData::get_age_slow@00661a00` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `ObjectTypeData::get_age_slow@00661a00`, run by `tools/emu/sweep_object_type_data_get_age_slow.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), vtable slot +0x4 (0 args) stubbed; callee 0x00668700 (2 args) stubbed; vtable slot +0x28 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; the word at `0xe85ddc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); the word at `0xc0aabc` points at 32 random bytes less `0` (placed); the word at `p9+0` points at 464 random bytes less `0` (placed); the word at `p9+4` points at 464 random bytes less `0` (placed); the word at `p9+8` points at 464 random bytes less `0` (placed); the word at `p9+12` points at 464 random bytes less `0` (placed); the word at `p9+16` points at 464 random bytes less `0` (placed); the word at `p9+20` points at 464 random bytes less `0` (placed); the word at `p9+24` points at 464 random bytes less `0` (placed); the word at `p9+28` points at 464 random bytes less `0` (placed); slot +0x28 laid as 0x004706c0 or its stub, a flag after the answers; slot +0x34 laid as 0x00470820 or its stub, a flag after the answers; stub 0 answered from [0, 1, 2, 3, 5] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 1 answers 4 values in call order, printed after the answers; stub 1 answers -1..7 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_get_age_slow.py") else {
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
    // get_age_slow: the first prerequisite (get_preq(i, -1), i below the type's count) that is a tech (index 0x220..0x274;
    // else the slot-0x28 answer) names the age: its age number from the tech record (+0x1c8), one more when it is an age
    // tech itself (0x220..0x226; else the slot-0x34 answer).
    let _ = (&c, &m, &a[1]);
    for i in 0..a[0] {
        let t = a[4 + (i as usize).min(3)];
        if t < 0 {
            continue;
        }
        let idx = rd(&p[1 + t as usize], 4);
        let tech = if a[8] != 0 { (0x220..0x275).contains(&idx) } else { a[2] != 0 };
        if !tech {
            continue;
        }
        let age = if a[9] != 0 { (0x220..0x227).contains(&idx) } else { a[3] != 0 };
        break 'g (rd(&p[10 + t as usize], 0x1c8) + age as i32) as i64;
    }
    0
}
        };
        assert_eq!(got, want_eax, "get_age_slow — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_age_slow — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_age_slow — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_age_slow — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectTypeData::get_age_slow@00661a00: {n} rows agree");
}
