//! `PathFinder::kill_lists@00687ae0` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::kill_lists@00687ae0`, run by `tools/emu/sweep_obj.py` on its row `path_finder_kill_lists` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), callee 0x00687c10 (1 args) stubbed; callee 0x00687ba0 (1 args) stubbed; callee 0x0046da60 (0 args) stubbed; callee 0x0046db10 (0 args) stubbed; callee 0x0046db80 (0 args) stubbed; callee 0x0046de10 (0 args) stubbed; callee 0x0046dc60 (0 args) stubbed; callee 0x00681e80 (0 args) stubbed; callee 0x0046dbf0 (0 args) stubbed; the word at `m+0x40` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `m+0x44` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `m+0x48` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `m+0x4c` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `m+0x50` points at 48 random bytes less `0` (placed, null a third of the rows); the word at `p0+0xc` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p2+0xc` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p3+0xc` points at 32 random bytes less `0` (placed, null a third of the rows); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 writes 0 (4 bytes) at `p3+0xc` each call; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_kill_lists") else {
        return;
    };
    let obj = super::Obj {
        name: "kill_lists",
        full: "PathFinder::kill_lists@00687ae0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Trees this+0x40/0x44/0x48/0x4c/0x50 are p[0..5]; the nodes at +0xc of the first, third and
            // fourth are p[5], p[6], p[7]. Every callee is hooked and logged by stub index and `ecx`
            // (and the node argument of the two kill_tree walks); remove_current (stub 6) clears
            // the fourth tree's +0xc, `--stub-write`, so its loop runs once.
            let (t40, t44, t48, t4c, t50) = (
                rd(m, 0x40),
                rd(m, 0x44),
                rd(m, 0x48),
                rd(m, 0x4c),
                rd(m, 0x50),
            );
            if t40 != 0 && rd(&p[0], 0xc) != 0 {
                log.push("s0:m+0x0,p5+0x0".to_string());
            }
            if t48 != 0 && rd(&p[2], 0xc) != 0 {
                log.push("s1:m+0x0,p6+0x0".to_string());
            }
            for (t, tok) in [
                (t40, "s2:p0+0x0"),
                (t44, "s3:p1+0x0"),
                (t48, "s4:p2+0x0"),
                (t50, "s5:p4+0x0"),
            ] {
                if t != 0 {
                    log.push(tok.to_string());
                }
            }
            if t4c != 0 {
                let head = rd(&p[3], 0xc);
                if head != 0 {
                    wr(&mut p[3], 0x14, 0);
                    wr(&mut p[3], 0x10, head);
                    let (v0, v1) = (rd(&p[7], 0xc), rd(&p[7], 0x10));
                    wr(&mut p[3], 0, v0);
                    wr(&mut p[3], 4, v1);
                }
                if rd(&p[3], 0xc) != 0 {
                    loop {
                        let cur = rd(&p[3], 0);
                        log.push("s6:p3+0x0".to_string());
                        wr(&mut p[3], 0xc, 0);
                        if cur != 0 {
                            log.push(format!("s7:{cur}"));
                        }
                        if rd(&p[3], 0xc) == 0 {
                            break;
                        }
                    }
                }
                log.push("s8:p3+0x0".to_string());
            }
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
        }
    });
}
