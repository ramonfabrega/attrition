//! `Signposts::head@00897b50` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Signposts::head@00897b50`, run by `tools/emu/sweep_obj.py` on its row `signposts_head` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 84 random bytes at 0x00eb1a64; callee 0x00a1eeb0 (1 args) stubbed; the word at `c0+0x20` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `c0+0x38` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `c0+0x50` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("signposts_head") else {
        return;
    };
    let obj = super::Obj {
        name: "head",
        full: "Signposts::head@00897b50",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Three lists' heads at 0xeb1a64 (c[0]): labels (head ptr +0x20, current +0x18,
            // length short +0x14), x (+0x38, +0x30, value +0x28, short +0x2c), y (+0x50,
            // +0x48, +0x40, +0x44); the nodes are p[0..3].
            let mut called = false;
            let head = rd(&c[0], 0x20);
            if head != 0 {
                wr(&mut c[0], 0x18, head);
                log.push("c0+0x0,p0+0x8".into());
                called = true;
                let (b0, b1) = (p[0][0x1c], p[0][0x1d]);
                c[0][0x14] = b0;
                c[0][0x15] = b1;
            }
            for (win, hd, cur, val, sh) in [
                (1usize, 0x38usize, 0x30usize, 0x28usize, 0x2cusize),
                (2, 0x50, 0x48, 0x40, 0x44),
            ] {
                let node = rd(&c[0], hd);
                if node != 0 {
                    wr(&mut c[0], cur, node);
                    let v = rd(&p[win], 8);
                    wr(&mut c[0], val, v);
                    let (b0, b1) = (p[win][0xc], p[win][0xd]);
                    c[0][sh] = b0;
                    c[0][sh + 1] = b1;
                }
            }
            if !called {
                log.push("-".into());
            }
            0
        }
    });
}
