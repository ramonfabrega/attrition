//! `Caravan::restart_trade_route@0073d070` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Caravan::restart_trade_route@0073d070`, run by `tools/emu/sweep_obj.py` on its row `caravan_restart_trade_route` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x0073de80 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x0073db10 (0 args) stubbed; the word at `0xc0aec0+i8(m,0xd)*0x1c` points at 4 random bytes less `max(i16(m,0xa),0)*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("caravan_restart_trade_route") else {
        return;
    };
    let obj = super::Obj {
        name: "restart_trade_route",
        full: "Caravan::restart_trade_route@0073d070",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Reset: the counter at +0x18 runs down to 0 (a negative or zero one is left at 0 too), the temp road is cleared (clear_temp_road, this), bit 2 of the flags byte +0xc dropped. A non-negative unit index (short +0xa) and non-negative player (signed byte +0xd): the unit is units[player].list[index]; with bit 0 of its byte +8 set it is repathed and the road rebuilt (build_road, this).
            wr(m, 0x18, 0);
            log.push("s0:m+0x0".into());
            m[0xc] &= 0xfb;
            let idx = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
            if idx >= 0 && (m[0xd] as i8) >= 0 && p[1][8] & 1 != 0 {
                log.push("s1:p1+0x0".into());
                log.push("s2:m+0x0".into());
            }
            0
        }
    });
}
