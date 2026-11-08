//! `Group::set_up_insert@0070e520` under the emulator against the port (item 3112; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::set_up_insert@0070e520`, run by `tools/emu/sweep_obj.py` on its row `group_set_up_insert` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 80 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p8 the place(s), in flag order; argument 0 names 32 random bytes; callee 0x0070ccb0 (1 args) stubbed; callee 0x0072f900 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `0xc0aec0` points at 4 random bytes less `0` (placed); the word at `p1+0` points at 256 random bytes less `0` (placed); the word at `p2+0xdc` points at 32 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p3+8` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); stub 0 answered from [-1, 0, 0, 0] (90%); the word at `p5+0` laid as the address of `p3` every row; stub 0 answers -1..0 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_set_up_insert") else {
        return;
    };
    let obj = super::Obj {
        name: "set_up_insert",
        full: "Group::set_up_insert@0070e520",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 300,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a`: 0 the list's address, 1 `find_leader`'s answer (-1 or 0), 2 `copy_order`'s answer, 3 `add`'s.
            // Windows: p2 the unit record, p3..p5 the ring's three nodes (`+0` next, `+8` order, `+0xc` flag byte),
            // p6..p8 their orders (flags at +4). The ring is laid by the alias, so the port walks it by index: the
            // head (`+0xdc`) is node 0, and the cursor (`+0xd4`) holds the node's own address, read back from the
            // window that points at it.
            let rd = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let wr = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            let mut calls: Vec<String> = Vec::new();
            if a[1] >= 0 && rd(&p[2], 0xd8) != 0 {
                let addr = [rd(&p[2], 0xdc), rd(&p[3], 0), rd(&p[4], 0)];
                let mut k = 0usize;
                let step = |p: &mut [Vec<u8>], k: usize| {
                    wr(&mut p[2], 0xd4, addr[k]);
                    let o = rd(&p[3 + k], 8);
                    wr(&mut p[2], 0xcc, o);
                    let f = p[3 + k][0xc];
                    p[2][0xd0] = f;
                };
                if rd(&p[2], 0xdc) != 0 {
                    step(p, 0);
                }
                loop {
                    if rd(&p[2], 0xcc) != 0 && p[6 + k][4] & 4 != 0 {
                        calls.push(format!("p0+0x4,{}", a[2]));
                    }
                    if rd(&p[2], 0xdc) != 0 {
                        k = (k + 1) % 3;
                        step(p, k);
                    }
                    if rd(&p[2], 0xd4) == rd(&p[2], 0xdc) {
                        break;
                    }
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        }
    });
}
