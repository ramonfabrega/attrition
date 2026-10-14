//! `ObjectData::is_cargo@00653600` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::is_cargo@00653600`, run by `tools/emu/sweep_object_data_is_cargo.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0xbc (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x12c (0 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); slot +0x12c laid as 0x00661d80 or its stub, a flag after the answers; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 5, -1] (90%); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_cargo.py") else {
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
            {
    // The answers of `this`'s slot `+0xbc` and the type's `+0xc` and
    // `+0x12c`, then whether `+0x12c` and `+0xc` were laid as the base ones.
    let (on_map, unit_ty, cargo) = (a[0], a[1], a[2]);
    let (cargo_base, unit_base) = (a[3] != 0, a[4] != 0);
    let land = rd(&p[0], 0x218) == 0;
    if on_map == 0 {
        0
    } else if !cargo_base {
        i64::from(cargo)
    } else if unit_base {
        i64::from(crate::transport::cargo_type(rd(&p[0], 4), land))
    } else {
        i64::from(unit_ty != 0 && land)
    }
}
        };
        assert_eq!(got, want_eax, "is_cargo — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_cargo — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_cargo — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_cargo — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::is_cargo@00653600: {n} rows agree");
}
