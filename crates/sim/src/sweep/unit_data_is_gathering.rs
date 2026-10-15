//! `UnitData::is_gathering@006089b0` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_gathering@006089b0`, run by `tools/emu/sweep_unit_data_is_gathering.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), 128 random bytes at 0x00c0aec0; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `c0+i8(m,9)*0x1c` points at 4 random bytes less `i16(m,0xa)*4` (placed); stub 1 answered from [7, 7, 7, 7, 0, 12] (90%); stub 2 answers the address of 16 random bytes, a `p` window after the places; stub 0 answers a window of its own to each call, in call order (8 bytes, ...); the stubbed vtable laid at `p2+0`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_gathering.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let ty = rd(&p[0], 4); let mut r = 0; if (0x32..=0x35).contains(&ty) && a[2] == 7 { r = i32::from(rd(&p[4], 0xc) == i32::from(m[9])); } i64::from(r)
        };
        assert_eq!(got, want_eax, "is_gathering — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_gathering — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_gathering — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "is_gathering — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_gathering@006089b0: {n} rows agree");
}
