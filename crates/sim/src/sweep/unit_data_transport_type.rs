//! `UnitData::transport_type@0046f790` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::transport_type@0046f790`, run by `tools/emu/sweep_unit_data_transport_type.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_transport_type.py") else {
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
                // `is(0x45, 0)` (scout) answers 1; a citizen kind (0x32..0x35), `is(0x3b,
                // 0)`, or the kinds 0x3d, 0x3e, 0x190 answer 3 (civilian); else 2. With
                // slot +0xb8 laid as `ObjectData::is` the type's slot +0x60 is asked
                // the same pushed pair (item 2281).
                let base = a[6] != 0;
                let ty = rd(&p[0], 4);
                let (mut own, mut typ) = (Vec::new(), Vec::new());
                let mut ask = |arg: i32, n: usize| -> i32 {
                    if base {
                        typ.push(format!("p0+0x0,{arg},0"));
                        a[4 + n]
                    } else {
                        own.push(format!("m+0x0,{arg},0"));
                        a[2 + n]
                    }
                };
                let r = if ask(0x45, 0) != 0 {
                    1
                } else if (0x32..=0x35).contains(&ty)
                    || ask(0x3b, 1) != 0
                    || matches!(ty, 0x3d | 0x3e | 0x190)
                {
                    3
                } else {
                    2
                };
                for calls in [own, typ] {
                    log.push(if calls.is_empty() {
                        "-".into()
                    } else {
                        calls.join("/")
                    });
                }
                r
            }
        };
        assert_eq!(got, want_eax, "transport_type — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "transport_type — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "transport_type — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "transport_type — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "transport_type — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::transport_type@0046f790: {n} rows agree");
}
