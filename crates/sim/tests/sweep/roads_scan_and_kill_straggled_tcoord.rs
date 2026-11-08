//! `Roads::scan_and_kill_straggled_tcoord@0088e050` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Roads::scan_and_kill_straggled_tcoord@0088e050`, run by `tools/emu/sweep_obj.py` on its row `roads_scan_and_kill_straggled_tcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1792 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s); the global cell 0x00c06218 points at 16 random bytes; the global cell 0x00c06188 points at 16 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x00866a70 (5 args) stubbed; callee 0x006b43b0 (5 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("roads_scan_and_kill_straggled_tcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "scan_and_kill_straggled_tcoord",
        full: "Roads::scan_and_kill_straggled_tcoord@0088e050",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 1200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let base = 0x650usize;
            let at = |i: i32| -> usize { (base as i32 + 4 * i) as usize };
            let (mut e, mut s, mut d) = (0, 0, 0);
            let mut kill = false;
            for k in 0..8 {
                let w = rd(m, at(k));
                if w != 0 && rd(m, at(k - 0x12)) == 0 {
                    kill = true;
                    break;
                }
                if w != 0 {
                    e += 1;
                }
                if rd(m, at(k + 9)) != 0 {
                    s += 1;
                }
                if rd(m, at(k - 9)) != 0 {
                    d += 1;
                }
            }
            if kill || (e == 1 && d == 0 && s == 0) {
                log.push(format!("s0:c0+0x0,{},{},0,0,1", rd(&p[0], 0), rd(&p[1], 0)));
                log.push("s1:c1+0x0,p0+0x0,p1+0x0,0,0,0".to_string());
            } else {
                log.push("-".to_string());
            }
            0
        }
    });
}
