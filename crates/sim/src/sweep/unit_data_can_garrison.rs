//! `UnitData::can_garrison@006099a0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::can_garrison@006099a0`, run by `tools/emu/sweep_unit_data_can_garrison.py` under unicorn
/// (tools/emu): 48 random bytes of object, 2 argument(s), vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x0061d7d0 (1 args) stubbed; the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); the word at `p2+0` points at 48 random bytes less `0` (placed); the word at `p2+4` points at 48 random bytes less `0` (placed); the word at `p3+0` points at 48 random bytes less `0` (placed); the word at `p3+4` points at 48 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `4` (placed); the word at `p5+0x18` points at 8 random bytes less `4` (placed); the word at `p6+0x18` points at 8 random bytes less `4` (placed); the word at `p7+0x18` points at 8 random bytes less `4` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); the stubbed vtable laid at `p4+0`; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_can_garrison.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
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
    // The object a[0] of leader a[1]'s list (the table at 0xc0ab84, rows 0x1c
    // apart): `p[4 + 2 * a[1] + a[0]]`, its record `p[8 + ...]`.
    let k = (2 * a[1] + a[0]) as usize;
    if p[4 + k][8] & 1 != 0 && a[2] != 0 && a[3] != 0 {
        log.push(format!("p12+0x0,{}", rd(&p[8 + k], 0)));
        i64::from(a[4])
    } else {
        log.push("-".into());
        0
    }
}
        };
        assert_eq!(got, want_eax, "can_garrison — eax, row `{line}`");
        assert_eq!(log, want_log, "can_garrison — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "can_garrison — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "can_garrison — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "can_garrison — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::can_garrison@006099a0: {n} rows agree");
}
