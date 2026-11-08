//! `GroupData::get_angle@0070c560` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::get_angle@0070c560`, run by `tools/emu/sweep_obj.py` on its row `group_data_get_angle` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2256 random bytes of object, 0 argument(s), callee 0x0070ccb0 (1 args) stubbed; the word at `0xc0aec0+u8(m,0x4a)*0x1c` points at 4 random bytes less `(s[0] if u8(m,0x49)==0 else i16(m,0x8cc))*4` (placed); the word at `p0+0` points at 4 random bytes less `0x50` (placed); stub 0 answered from [0, 1, 2, 5, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..7 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_get_angle") else {
        return;
    };
    let obj = super::Obj {
        name: "get_angle",
        full: "GroupData::get_angle@0070c560",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Members (+0xc) none → 0. A building group (+0x49) leads with its first member
            // (+0x8cc, a short) and angle 0; else `find_leader` (stub 0) answers the leader and
            // writes its angle's index (`o`) into a local. A leader → its unit's angle (+0x50;
            // the units of player +0x4a through the table at 0xc0aec0) less that angle (+0x84c,
            // a signed byte a member) shifted up 24; a negative leader → 0.
            if rd(m, 0xc) <= 0 {
                log.push("-".into());
                0
            } else {
                let (idx, k) = if m[0x49] != 0 {
                    log.push("-".into());
                    (i32::from(i16::from_le_bytes([m[0x8cc], m[0x8cd]])), 0)
                } else {
                    log.push("m+0x0,sp-0xc".into());
                    (a[0], a[1] as usize)
                };
                if idx < 0 {
                    0
                } else {
                    i64::from(rd(&p[1], 0).wrapping_sub(i32::from(m[0x84c + k] as i8) << 24))
                }
            }
        }
    });
}
