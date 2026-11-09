//! `WorldMap::reinit_all@008d2b60` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldMap::reinit_all@008d2b60`, run by `tools/emu/sweep_obj.py` on its row `world_map_reinit_all` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 4 random bytes at 0x00e399ac; vtable slot +0x420 (0 args) stubbed; vtable slot +0x18 (1 args) stubbed; callee 0x008d2cd0 (0 args) stubbed; callee 0x008d28d0 (0 args) stubbed; the word at `c0+0` points at 16 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_map_reinit_all") else {
        return;
    };
    let obj = super::Obj {
        name: "reinit_all",
        full: "WorldMap::reinit_all@008d2b60",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The world map (the global cell at 0xe399ac, a pointer laid into c[0], null a third of the rows) is
            // reinitialised (stub 2), then asked through its vtable's slot +0x420 (stub 0) and +0x18 (stub 1) with
            // 0; the fog's init (stub 3, whose answer is the result) is called either way, handed `ecx` untouched
            // by the null path, else the map (a stub leaves `ecx` as it found it).
            if rd(&c[0], 0) == 0 {
                log.push("s3:m+0x0".to_string());
            } else {
                log.push("s2:m+0x0".to_string());
                log.push("s0:p0+0x0".to_string());
                log.push("s1:p0+0x0,0".to_string());
                log.push("s3:p0+0x0".to_string());
            }
            i64::from(a[3])
        }
    });
}
