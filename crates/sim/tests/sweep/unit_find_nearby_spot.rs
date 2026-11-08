//! `Unit::find_nearby_spot@00617010` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::find_nearby_spot@00617010`, run by `tools/emu/sweep_obj.py` on its row `unit_find_nearby_spot` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 48 random bytes of object, 14 argument(s), callee 0x0061de70 (16 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_find_nearby_spot") else {
        return;
    };
    let obj = super::Obj {
        name: "find_nearby_spot",
        full: "Unit::find_nearby_spot@00617010",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `Unit::find_nearby_spot(a0..a13)`: forwards to `UnitType::find_nearby_spot` (`0x61de70`) on the type
        // record at `this+0x18` (`p[0]`), the first nine arguments, then the unit's short at `+0xa`
        // (sign-extended) and byte at `+9`, then `a9` and `a10`, `-1`, `0` and `a13`; its answer is eax.
        let (w, b) = (i16::from_le_bytes([m[0xa], m[0xb]]) as i32, m[9] as i32);
        let mut v: Vec<i32> = a[..9].to_vec();
        v.extend([w, b, a[9], a[10], -1, 0, a[13]]);
        let args: Vec<String> = v.iter().map(|x| x.to_string()).collect();
        log.push(format!("p0+0x0,{}", args.join(",")));
        i64::from(a[14])
    });
}
