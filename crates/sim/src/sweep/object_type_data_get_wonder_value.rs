//! `ObjectTypeData::get_wonder_value@00661ba0` under the emulator against the port (item 3120; tools/sweep_gen.py, obj with windows).

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

/// `ObjectTypeData::get_wonder_value@00661ba0`, run by `tools/emu/sweep_object_type_data_get_wonder_value.py` under unicorn
/// (tools/emu): 56 random bytes of object, 0 argument(s), `p` windows in order: p0..p10 the place(s), in flag order; the global cell 0x00c061e8 points at 2304 random bytes; 2 random bytes at 0x00ee10a7 (each byte in 0..1); vtable slot +0x1c (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; callee 0x00a1edd0 (1 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; the word at `0xc0aaa0` points at 2184 random bytes less `0` (placed); the word at `p0+0x834` points at 724 random bytes less `0` (placed); the word at `p0+0x838` points at 724 random bytes less `0` (placed); the word at `p0+0x850` points at 724 random bytes less `0` (placed); the word at `p0+0x878` points at 724 random bytes less `0` (placed); the word at `p0+0x87c` points at 724 random bytes less `0` (placed); the word at `0xe85ddc` points at 20 random bytes less `-4` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p6+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 16 random bytes less `0` (placed); the word at `p6+12` points at 16 random bytes less `0` (placed); slot +0x1c laid as 0x00470780 or its stub, a flag after the answers; slot +0x34 laid as 0x00470820 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 4 answers 0..0 every row; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_get_wonder_value.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // `a`: 0 and 1 the answers of slots +0x1c (is_wonder_type) and +0x34 (is_age_type), 2..5 the String and Error::report stubs'
    // (report answers 0 every row: a nonzero answer is the original's own `int3`), 6 and 7 the base flags of +0x1c and +0x34.
    // Windows: c0 the gamec record, c1 the two `ignore_always` bytes, p0 the buildtypes array, p1..p5 its records at this+4 in
    // {0x20d, 0x20e, 0x214, 0x21e, 0x21f}, p6 the types list's block, p7..p10 its entries (ebx -1..2).
    let rd = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
    let ty = rd(m, 4);
    let wonder = if a[6] != 0 { (0x20e..0x21f).contains(&ty) } else { a[0] != 0 };
    if !wonder {
        0
    } else {
        let ebx = rd(m, 0x30);
        if ebx < 0 && (c[0][0x821] & 0x10 != 0 || c[0][0x822] & 2 != 0) {
            0
        } else {
            // the first `Error::report` arm (ebx < 0, `ignore_always` clear) and the second (the prerequisite is not an age
            // type) report through stubs and fall through either way.
            let age = rd(&p[7 + (ebx + 1) as usize], 4);
            let _is_age = if a[7] != 0 { (0x220..0x227).contains(&age) } else { a[1] != 0 };
            let at = match ty {
                0x20d => 1,
                0x20e => 2,
                0x214 => 3,
                0x21e => 4,
                _ => 5,
            };
            i64::from(rd(&p[at], 0x2d0))
        }
    }
}
        };
        assert_eq!(got, want_eax, "get_wonder_value — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_wonder_value — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_wonder_value — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "get_wonder_value — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("ObjectTypeData::get_wonder_value@00661ba0: {n} rows agree");
}
