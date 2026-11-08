//! `Map::stamp@0069cf80` under the emulator against the port (item 2831; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Map::stamp@0069cf80`, run by `tools/emu/sweep_obj.py` on its row `map_stamp` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 3 argument(s), callee 0x0069d5a0 (3 args) stubbed; stub 0 answered from [0, 1, 5] (90%); stub 0 answers 5 values in call order, the last from [0, 1, 5], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_stamp") else {
        return;
    };
    let obj = super::Obj {
        name: "stamp",
        full: "Map::stamp@0069cf80",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Map::point (stub 0) answers a[4..9] in call order; the first zero ends it.
        let calls = [(0, 0), (1, 0), (-1, 0), (0, -1), (0, 1)];
        let mut res = 1;
        for (i, (dx, dy)) in calls.iter().enumerate() {
            log.push(format!(
                "s0:m+0x0,{},{},{}",
                a[0],
                a[1].wrapping_add(*dx),
                a[2].wrapping_add(*dy)
            ));
            if a[4 + i] == 0 {
                res = 0;
                break;
            }
        }
        res
    });
}
