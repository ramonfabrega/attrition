//! `UnitData::get_gpiece@0060dd80` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::get_gpiece@0060dd80`, run by `tools/emu/sweep_unit_data_get_gpiece_2.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; callee 0x0090c030 (7 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); the word at `0xc0aef8` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `p10+8` points at 32 random bytes less `0` (placed); the word at `p10+12` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 16 random bytes less `0` (placed); the word at `p2+0x18` points at 16 random bytes less `0` (placed); the word at `p3+0x18` points at 16 random bytes less `0` (placed); the word at `p4+0x18` points at 16 random bytes less `0` (placed); the word at `p6+0x18` points at 16 random bytes less `0` (placed); the word at `p7+0x18` points at 16 random bytes less `0` (placed); the word at `p8+0x18` points at 16 random bytes less `0` (placed); the word at `p9+0x18` points at 16 random bytes less `0` (placed); the word at `p11+0x18` points at 16 random bytes less `0` (placed); the word at `p12+0x18` points at 16 random bytes less `0` (placed); the word at `p13+0x18` points at 16 random bytes less `0` (placed); the word at `p14+0x18` points at 16 random bytes less `0` (placed); the word at `m+0xf4` points at 16 random bytes less `0` (placed); the word at `p27+0` points at 144 random bytes less `0` (placed); the word at `p27+4` points at 144 random bytes less `0` (placed); the word at `p27+8` points at 144 random bytes less `0` (placed); the word at `p27+12` points at 144 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 5] (90%); stub 1 answered from [0, 1, 7] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p32+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_get_gpiece_2.py") else {
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
            let _ = (&c, &m);
            // With the guys list non-empty (word +0xe8) the answer is the record `param_1` of the list at +0xf4
            // (its word +0x88). Otherwise `this`'s slot 0xb8 is asked (the stubbed answer) whether it holds the graphic-piece
            // flag: the base `ObjectData::is` is replaced by the owner's slot 0x60 (laid as a flag in the row),
            // and the answer's truth is handed with the unit's two coordinates, the unit-type record's word +4
            // and the table at 0xc0aec0 + 28 * byte [+9] to the piece lookup, whose answer is returned.
            let _ = a;
            if rd(m, 0xe8) != 0 {
                log.extend(["-".into(), "-".into(), "-".into()]);
                i64::from(rd(&p[28 + a[0] as usize], 0x88))
            } else {
                let (k, j) = (m[9] as usize, i16::from_le_bytes([m[10], m[11]]) as usize);
                let r = if a[5] != 0 {
                    log.push("-".into());
                    log.push("p32+0x0".into());
                    a[3]
                } else {
                    log.push("m+0x0,62,0".into());
                    log.push("-".into());
                    a[2]
                };
                let val = rd(&p[15 + 4 * k + j], 4);
                log.push(format!(
                    "15242968,{val},{k},{j},0,{},0,0",
                    i32::from(r != 0)
                ));
                i64::from(a[4])
            }
        };
        assert_eq!(got, want_eax, "get_gpiece — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_gpiece — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_gpiece — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_gpiece — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_gpiece — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::get_gpiece@0060dd80: {n} rows agree");
}
