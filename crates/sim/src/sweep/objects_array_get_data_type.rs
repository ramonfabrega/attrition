//! `ObjectsArray::get_data_type@00475640` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsArray::get_data_type@00475640`, run by `tools/emu/sweep_objects_array_get_data_type.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; 4 random bytes at 0x00ee10a0; argument 0 names 16 random bytes (null a third of the rows); vtable slot +0x30 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_array_get_data_type.py") else {
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
            // `a[0]` the object (0: none), `a[1..5]` the answers of its slots 0x30, 0x18, 0x20, 0x1c
            // in the order they are asked; the first nonzero names the kind (3, 0, 1, 2); none left
            // the error report, skipped here by the `ignore_always` byte at `c0+1`, and -1.
            let kinds = [3, 0, 1, 2];
            let mut calls = vec!["-"; 4];
            let mut got = -1;
            if a[0] != 0 {
                for (i, k) in kinds.iter().enumerate() {
                    calls[i] = "p0+0x0";
                    if a[1 + i] != 0 {
                        got = *k;
                        break;
                    }
                }
                assert!(
                    got >= 0 || c[0][1] != 0,
                    "the error report arm is not drawn"
                );
            }
            log = calls.iter().map(|s| s.to_string()).collect();
            got
        };
        assert_eq!(got, want_eax, "get_data_type — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_data_type — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_data_type — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_data_type — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_data_type — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectsArray::get_data_type@00475640: {n} rows agree");
}
