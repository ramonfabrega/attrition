//! `GroupData::equals_group_const@0070cdf0` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::equals_group_const@0070cdf0`, run by `tools/emu/sweep_obj.py` on its row `group_data_equals_group_const` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2268 random bytes of object, 1 argument(s), argument 0 names 2268 random bytes; callee 0x0070d000 (0 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_equals_group_const") else {
        return;
    };
    let obj = super::Obj {
        name: "equals_group_const",
        full: "GroupData::equals_group_const@0070cdf0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The owner byte (+0x4a) first; then `get_num_const` (stub 0) of the other group, then
            // of this one, equal, the building flag (+0x49) equal, and the first `num` (+0xc) unit
            // indices of the lists (+0x8cc, shorts) equal.
            let q = &p[0];
            if m[0x4a] != q[0x4a] {
                log.push("-".into());
                0
            } else {
                log.push("p0+0x0/m+0x0".into());
                let same = a[3] == a[2]
                    && m[0x49] == q[0x49]
                    && (0..rd(m, 0xc).max(0) as usize).all(|i| {
                        m[0x8cc + 2 * i..0x8ce + 2 * i] == q[0x8cc + 2 * i..0x8ce + 2 * i]
                    });
                i64::from(same)
            }
        }
    });
}
