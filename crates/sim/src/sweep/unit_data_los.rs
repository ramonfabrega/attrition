//! `UnitData::los@006100c0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::los@006100c0`, run by `tools/emu/sweep_unit_data_los.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), vtable slot +0x10c (0 args) stubbed; callee 0x00646b00 (2 args) stubbed; the word at `0xc061e0` points at 64 random bytes less `0x59c0` (placed); the word at `0xc061e4` points at 336 random bytes less `0xb60` (placed); the word at `m+0x18` points at 720 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p2+0`; stub 1 answers -1..1 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_los.py") else {
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
            // `UnitData::los`: the unit's own sight (`+0x3c`, a signed byte) plus two bonuses. For a player
            // (`+9`, held to 0; `p[0]`) with the short at `+0x59d2` set, and a type (`p[2]`) whose `+0x2c8`
            // has `0x420`, a general `0x16a` (`has_general`, `0x646b00`; negative = none) adds the
            // constant at `+0xb64` (`p[1]`). With the short at `+0x59c8` set and the type's slot `0x10c`
            // answering 0, a general `0x165` adds the constant at `+0xca8`. The log holds each
            // `has_general`'s arguments (`0`, then the general).
            let (s0, s1) = (a[0], a[1]);
            let mut v = i32::from(m[0x3c] as i8);
            let mut calls: Vec<String> = Vec::new();
            if i16::from_le_bytes([p[0][0x12], p[0][0x13]]) != 0 && rd(&p[2], 0x2c8) & 0x420 != 0 {
                calls.push("m+0x0,0,362".to_string());
                if s1 >= 0 {
                    v = v.wrapping_add(rd(&p[1], 4));
                }
            }
            if i16::from_le_bytes([p[0][8], p[0][9]]) != 0 && s0 == 0 {
                calls.push(format!("m+0x0,{s0},357"));
                if s1 >= 0 {
                    v = v.wrapping_add(rd(&p[1], 0x148));
                }
            }
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            i64::from(v)
        };
        assert_eq!(got, want_eax, "los — eax, row `{line}`");
        assert_eq!(log, want_log, "los — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "los — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "los — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "los — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::los@006100c0: {n} rows agree");
}
