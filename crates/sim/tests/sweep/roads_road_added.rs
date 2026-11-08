//! `Roads::road_added@008954d0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Roads::road_added@008954d0`, run by `tools/emu/sweep_obj.py` on its row `roads_road_added` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 4 argument(s), the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00c06218 points at 32 random bytes; the global cell 0x00c0620c points at 576 random bytes; callee 0x00866a70 (5 args) stubbed; callee 0x004a3af0 (1 args) stubbed; callee 0x0088f4b0 (1 args) stubbed; the word at `m+0xf0` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("roads_road_added") else {
        return;
    };
    let obj = super::Obj {
        name: "road_added",
        full: "Roads::road_added@008954d0",
        cells: 3,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let w = rd(&c[0], 0);
            let (x, y) = (a[0], a[1]);
            if a[2] != 0 {
                log.push(format!("s0:c1+0x0,{x},{y},1,1,0"));
            }
            let (n, cap) = (rd(m, 0xe4), rd(m, 0xe8));
            if n >= cap {
                let s = i16::from_le_bytes([m[0xec], m[0xed]]);
                log.push(format!("s1:m+0xe0,{s}"));
            }
            let at = 8 * n as usize;
            wr(
                &mut p[0],
                at,
                x.wrapping_add(w.wrapping_mul(y).wrapping_mul(4)),
            );
            wr(&mut p[0], at + 4, 0x101);
            wr(m, 0xe4, n + 1);
            if a[3] != 0 {
                log.push(format!("s2:m+0x0,{}", i32::from(a[2] != 0)));
            }
            c[2][0x22c] = 1;
            if log.is_empty() {
                log.push("-".into());
            }
            0
        }
    });
}
