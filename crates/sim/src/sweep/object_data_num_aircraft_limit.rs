//! `ObjectData::num_aircraft_limit@006454a0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::num_aircraft_limit@006454a0`, run by `tools/emu/sweep_object_data_num_aircraft_limit.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 176 random bytes; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_num_aircraft_limit.py") else {
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
            {
                // A carrier (0x15f) and an airbase (0x1bf) answer their constants; else
                // whether it is 0x208. With slot +0xb8 laid as `ObjectData::is` the
                // type's slot +0x60 is asked the same pushed pair (item 2281).
                let base = a[8] != 0;
                let (mut own, mut typ) = (Vec::new(), Vec::new());
                let mut ask = |arg: i32, n: usize| -> i32 {
                    if base {
                        typ.push(format!("p0+0x0,{arg},1"));
                        a[5 + n]
                    } else {
                        own.push(format!("m+0x0,{arg},1"));
                        a[2 + n]
                    }
                };
                let r = if ask(0x15f, 0) != 0 {
                    rd(&c[0], 0xa4)
                } else if ask(0x1bf, 1) != 0 {
                    rd(&c[0], 0xa8)
                } else {
                    i32::from(ask(0x208, 2) != 0)
                };
                for calls in [own, typ] {
                    log.push(if calls.is_empty() {
                        "-".into()
                    } else {
                        calls.join("/")
                    });
                }
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "num_aircraft_limit — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "num_aircraft_limit — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "num_aircraft_limit — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "num_aircraft_limit — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "num_aircraft_limit — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::num_aircraft_limit@006454a0: {n} rows agree");
}
