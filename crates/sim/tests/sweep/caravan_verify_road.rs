//! `Caravan::verify_road@0073d950` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Caravan::verify_road@0073d950`, run by `tools/emu/sweep_obj.py` on its row `caravan_verify_road` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x0073db10 (0 args) stubbed; the word at `0xc06188` points at 320 random bytes less `0` (placed); the word at `p0+0x138` points at 64 random bytes less `0` (placed); the word at `0xcae5fc` points at 32 random bytes less `0` (placed); the word at `m+0x10` points at 48 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("caravan_verify_road") else {
        return;
    };
    let obj = super::Obj {
        name: "verify_road",
        full: "Caravan::verify_road@0073d950",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `Caravan::verify_road`: walks the `+0x18` waypoints (16 bytes each at `[+0x10]`, `x` then `y`);
        // a waypoint's tile word is `tiles[div3[y >> 6] * width + div3[x >> 6]]` (`p[1]`, with the
        // `0xcae5fc` table `p[2]` and the world's width at `p[0]+0x18`); the first whose low byte is
        // negative or has both low bits set sends the caravan to `build_road` (`0x73db10`, `ecx` this).
        let mut call = "-".to_string();
        let width = rd(&p[0], 0x18);
        for i in 0..rd(m, 0x18) as usize {
            let (x, y) = (rd(&p[3], 16 * i), rd(&p[3], 16 * i + 4));
            let idx = rd(&p[2], 4 * (y >> 6) as usize) * width + rd(&p[2], 4 * (x >> 6) as usize);
            let b = p[1][2 * idx as usize];
            if (b as i8) < 0 || b & 3 == 3 {
                call = "m+0x0".to_string();
                break;
            }
        }
        log.push(call);
        0i64
    });
}
