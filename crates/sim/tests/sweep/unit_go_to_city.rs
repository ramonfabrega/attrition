//! `Unit::go_to_city@005f79c0` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::go_to_city@005f79c0`, run by `tools/emu/sweep_obj.py` on its row `unit_go_to_city` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 3 argument(s), the global cell 0x00c0618c points at 256 random bytes; 16 random bytes at 0x00c0998c; callee 0x0046cff0 (0 args) stubbed; callee 0x005f7a50 (5 args) stubbed; the word at `c1+0` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+0xc` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 64 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+0xc` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1151, 1152, 1153, -5, 5000] (90%); stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_go_to_city") else {
        return;
    };
    let obj = super::Obj {
        name: "go_to_city",
        full: "Unit::go_to_city@005f79c0",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Player 1'"'"'s city a0 (the cities row at 0xc0998c: its pointer array, each record'"'"'s short at +8 a unit index into the objects table'"'"'s player-1 array at +0x30): when the unit is more than 0x480 away from this (positions xor 0x63637; vector_dist the stub, answered), this goes to it (x, y, a2, 0x240, 0); edx the distance'"'"'s y.
            let city = a[0] as usize;
            let idx = i16::from_le_bytes([p[1 + city][8], p[1 + city][9]]) as usize;
            let unit = &p[6 + idx];
            let ux = rd(unit, 0x10) ^ 0x63637;
            let uy = rd(unit, 0x14) ^ 0x63637;
            let tx = rd(m, 0x10) ^ 0x63637;
            let ty = rd(m, 0x14) ^ 0x63637;
            let dy = uy.wrapping_sub(ty);
            log.push(format!("{},{dy}", ux.wrapping_sub(tx)));
            log.push(if a[3] > 0x480 {
                format!("m+0x0,{dy},{ux},{uy},{},576,0", a[2])
            } else {
                "-".into()
            });
            0
        }
    });
}
