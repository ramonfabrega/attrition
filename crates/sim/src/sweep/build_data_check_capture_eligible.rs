//! `BuildData::check_capture_eligible@0062d1d0` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::check_capture_eligible@0062d1d0`, run by `tools/emu/sweep_build_data_check_capture_eligible.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x64 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x118 (0 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 3 answered from [3, 4, 5, 6, 7] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_check_capture_eligible.py") else {
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
            // The type (`m+0x18`) is a city: its slot +0x64 unless that is `BuildTypeData::is_city`,
            // which the caller inlines as slot +0x60 handed `(0x19e, 0)`. Then the building is active: its
            // slot +0x4c unless that is `WallData::is_active`, inlined as `+8` bit 2. Then slot +0x118
            // against 5 (`+8` bit 5 set) or 4 (item 2837).
            let city = if a[4] != 0 {
                log.push("s1:p0+0x0,414,0".into());
                i64::from(a[1])
            } else {
                log.push("s0:p0+0x0".into());
                i64::from(a[0])
            };
            let mut ans = 0;
            if city != 0 {
                let active = if a[5] != 0 {
                    i64::from(m[8] & 4)
                } else {
                    log.push("s2:m+0x0".into());
                    i64::from(a[2])
                };
                if active != 0 {
                    log.push("s3:m+0x0".into());
                    let level = i64::from(a[3]);
                    ans = i64::from(level > if m[8] & 0x20 != 0 { 5 } else { 4 });
                }
            }
            ans
        };
        assert_eq!(got, want_eax, "check_capture_eligible — eax, row `{line}`");
        assert_eq!(log, want_log, "check_capture_eligible — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_capture_eligible — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_capture_eligible — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_capture_eligible — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::check_capture_eligible@0062d1d0: {n} rows agree");
}
