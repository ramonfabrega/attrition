//! `ObjectTypeData::is_slow@00661ae0` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

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

/// `ObjectTypeData::is_slow@00661ae0`, run by `tools/emu/sweep_object_type_data_is_slow.py` under unicorn
/// (tools/emu): 608 random bytes of object, 2 argument(s), vtable slot +0xc (0 args) stubbed; vtable slot +0xf4 (2 args) stubbed; the word at `0xc0a274` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 4 random bytes less `0x2b4` (placed); the word at `0xc0aaf4` points at 4 random bytes less `i32(m,0x3c)*4` (placed); the word at `p2+0` points at 8 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, 2] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_is_slow.py") else {
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
                // The type itself (+4) is a0 -> 1. With a1: a unit type (slot +0xc,
                // inlined as 0x32 <= id < 0x19e when it is TypeData::is_unit_type) whose
                // graft (+0x25c) is a0, a0's unit type without +0x2b4 bit 0x1000000 -> 1.
                // Without: a0 >= 0, the graft a0 -> 1, else the type at +0x3c (when not
                // negative) asked through its slot +0xf4 (a0, 0).
                let id = rd(m, 4);
                let (based, s0, s1) = (a[4] != 0, a[2], a[3]);
                let mut l0 = "-".to_string();
                let mut l1 = "-".to_string();
                let v = if id == a[0] {
                    1
                } else if a[1] != 0 {
                    let unit = if based {
                        (0x32..0x19e).contains(&id)
                    } else {
                        l0 = "m+0x0".into();
                        s0 != 0
                    };
                    i32::from(unit && rd(m, 0x25c) == a[0] && rd(&p[1], 0) & 0x0100_0000 == 0)
                } else if a[0] < 0 {
                    0
                } else if rd(m, 0x25c) == a[0] {
                    1
                } else if rd(m, 0x3c) < 0 {
                    0
                } else {
                    l1 = format!("p3+0x0,{},0", a[0]);
                    s1
                };
                log.push(l0);
                log.push(l1);
                i64::from(v)
            }
        };
        assert_eq!(got, want_eax, "is_slow — eax, row `{line}`");
        assert_eq!(log, want_log, "is_slow — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_slow — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_slow — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_slow — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectTypeData::is_slow@00661ae0: {n} rows agree");
}
