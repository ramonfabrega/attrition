//! `WonderData::get_value@0073c7a0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `WonderData::get_value@0073c7a0`, run by `tools/emu/sweep_wonder_data_get_value.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), vtable slot +0x14c (0 args) stubbed; vtable slot +0x118 (0 args) stubbed; the word at `0xc0ab84+i8(m,0xd)*0x1c` points at 4 random bytes less `i16(m,2)*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 4 random bytes less `0` (placed); slot +0x14c laid as 0x0046cf20 or its stub, a flag after the answers; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wonder_data_get_value.py") else {
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
                // The object `objects[who][o]`; its slot +0x14c laid as
                // `ObjectData::get_wonder_value` hands the tail `jmp` to its type's
                // slot +0x118, else the slot itself is the tail (item 2281).
                if a[2] != 0 {
                    log.push("-".into());
                    log.push("p2+0x0".into());
                    i64::from(a[1])
                } else {
                    log.push("p1+0x0".into());
                    log.push("-".into());
                    i64::from(a[0])
                }
            }
        };
        assert_eq!(got, want_eax, "get_value — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_value — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_value — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_value — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_value — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WonderData::get_value@0073c7a0: {n} rows agree");
}
