//! `BuildData::check_capture_eligible@0062d1d0` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::check_capture_eligible@0062d1d0`, run by `tools/emu/sweep_obj.py` on its row `build_data_check_capture_eligible` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x64 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x118 (0 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 3 answered from [3, 4, 5, 6, 7] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_check_capture_eligible") else {
        return;
    };
    let obj = super::Obj {
        name: "check_capture_eligible",
        full: "BuildData::check_capture_eligible@0062d1d0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
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
    });
}
