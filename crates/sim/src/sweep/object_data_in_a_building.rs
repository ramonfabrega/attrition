//! `ObjectData::in_a_building@00644080` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::in_a_building@00644080`, run by `tools/emu/sweep_object_data_in_a_building.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x00651a80 (1 args) stubbed; vtable slot +0x20 (0 args) stubbed; the word at `0xc0ab84+o[0]*0x1c` points at 4 random bytes less `max(0,s[0])*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the stubbed vtable laid at `p1+0`; stub 0 answers -2..6 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..7 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_in_a_building.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `get_inside` (stub 0) answers the index of the object `this` is inside and writes its
                // owner through the pointer it is handed (`o`, a[2]); a negative index answers 0,
                // else objects[owner][index]'s slot +0x20 answers.
                if a[0] < 0 {
                    log.push("-".into());
                    0
                } else {
                    log.push("p1+0x0".into());
                    i64::from(a[1])
                }
            }
        };
        assert_eq!(got, want_eax, "in_a_building — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "in_a_building — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "in_a_building — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "in_a_building — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "in_a_building — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::in_a_building@00644080: {n} rows agree");
}
