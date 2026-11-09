//! `PathFinder::valid_ucoord@00687c80` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::valid_ucoord@00687c80`, run by `tools/emu/sweep_obj.py` on its row `path_finder_valid_ucoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 152 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00cae5fc points at 128 random bytes; callee 0x00479b90 (1 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x00617060 (7 args) stubbed; callee 0x00479ef0 (2 args) stubbed; the word at `m+0x50` points at 16 random bytes less `0` (placed); the word at `m+0x54` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_valid_ucoord") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_ucoord",
        full: "PathFinder::valid_ucoord@00687c80",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let (x, y, key) = (a[0], a[1], a[2]);
            let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            if x < 0 || y < 0 || x >= w.wrapping_mul(192) || y >= h.wrapping_mul(192) {
                log.push("-".into());
                0
            } else {
                log.push(format!("s0:p0+0x0,{key}"));
                if a[3] != 0 {
                    let n = rd(m, 0x90);
                    wr(m, 0x90, n.wrapping_add(1));
                    i64::from(rd(&p[0], 0) != 0)
                } else {
                    let tx = rd(&c[1], 4 * (x >> 6) as usize);
                    let ty = rd(&c[1], 4 * (y >> 6) as usize);
                    log.push(format!("s1:p1+0x0,{tx},{ty},0,1,0,1,0,{}", y >> 6));
                    let mut r = 0;
                    if a[4] == 0 {
                        log.push(format!("s2:p1+0x0,{x},{y},1,1,p1+0x0,1,0"));
                        r = i64::from(a[5] == 0);
                    }
                    log.push(format!("s3:p0+0x0,{r},{key}"));
                    r
                }
            }
        }
    });
}
