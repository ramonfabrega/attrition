//! `BuildTypeData::blocked_town@00636d80` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::blocked_town@00636d80`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_blocked_town` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 4 argument(s), callee 0x00636a50 (5 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_blocked_town") else {
        return;
    };
    let obj = super::Obj {
        name: "blocked_town",
        full: "BuildTypeData::blocked_town@00636d80",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A forwarder: `blocked_site(this, x * 0x300, y * 0x300, a2, -1, 0)` (stub 0), its answer.
            log.push(format!(
                "m+0x0,{},{},{},-1,0",
                a[0].wrapping_mul(0x300),
                a[1].wrapping_mul(0x300),
                a[2]
            ));
            i64::from(a[4])
        }
    });
}
