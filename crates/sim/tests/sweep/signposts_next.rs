//! `Signposts::next@00897ac0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Signposts::next@00897ac0`, run by `tools/emu/sweep_obj.py` on its row `signposts_next` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 84 random bytes at 0x00eb1a64; callee 0x00a1eeb0 (1 args) stubbed; the word at `c0+0x18` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 8 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `c0+0x48` points at 8 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("signposts_next") else {
        return;
    };
    let obj = super::Obj {
        name: "next",
        full: "Signposts::next@00897ac0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The three lists' next nodes: labels (guard +0x20, current +0x18, short +0x14,
            // the call's argument the node + 8), x (guard +0x38, current +0x30, value +0x28,
            // short +0x2c), y (guard +0x50, +0x48, +0x40, +0x44).
            if rd(&c[0], 0x20) != 0 {
                let next = rd(&p[0], 0);
                wr(&mut c[0], 0x18, next);
                log.push("c0+0x0,p1+0x8".into());
                let (b0, b1) = (p[1][0x1c], p[1][0x1d]);
                c[0][0x14] = b0;
                c[0][0x15] = b1;
            } else {
                log.push("-".into());
            }
            for (src, node, guard, cur, val, sh) in [
                (2usize, 3usize, 0x38usize, 0x30usize, 0x28usize, 0x2cusize),
                (4, 5, 0x50, 0x48, 0x40, 0x44),
            ] {
                if rd(&c[0], guard) != 0 {
                    let next = rd(&p[src], 0);
                    wr(&mut c[0], cur, next);
                    let v = rd(&p[node], 8);
                    wr(&mut c[0], val, v);
                    let (b0, b1) = (p[node][0xc], p[node][0xd]);
                    c[0][sh] = b0;
                    c[0][sh + 1] = b1;
                }
            }
            0
        }
    });
}
