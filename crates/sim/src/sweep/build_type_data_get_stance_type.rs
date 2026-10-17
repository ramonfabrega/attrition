//! `BuildTypeData::get_stance_type@006396c0` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::get_stance_type@006396c0`, run by `tools/emu/sweep_build_type_data_get_stance_type.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), callee 0x00639de0 (0 args) stubbed; vtable slot +0x64 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_get_stance_type.py") else {
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
            // Not a training building (`a[0]`) is `-1`; the city slot (`a[1]`) is 1 (worker), the fort slot
            // (`a[2]`) 2 (caster); the type word `m+4` 0x1ae/0x1af is 3 (packer), 0x1b4/0x1a4/0x1bf/0x208
            // is -1, any other 0 (item 2844). The slots are stubs here, never `is_city`/`is_fort`.
            let id = rd(m, 4);
            if a[0] == 0 {
                -1
            } else if a[1] != 0 {
                1
            } else if a[2] != 0 {
                2
            } else if id == 0x1ae || id == 0x1af {
                3
            } else if id == 0x1b4 || id == 0x1a4 || id == 0x1bf || id == 0x208 {
                -1
            } else {
                0
            }
        };
        assert_eq!(got, want_eax, "get_stance_type — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_stance_type — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_stance_type — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_stance_type — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::get_stance_type@006396c0: {n} rows agree");
}
