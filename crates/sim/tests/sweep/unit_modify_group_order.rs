//! `Unit::modify_group_order@005e3530` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::modify_group_order@005e3530`, run by `tools/emu/sweep_obj.py` on its row `unit_modify_group_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: none; 16 random bytes at 0x00e90000; 16 random bytes at 0x00e90010; 16 random bytes at 0x00e90020; 16 random bytes at 0x00e90030; 64 random bytes at 0x00e90100; 64 random bytes at 0x00e90180; 64 random bytes at 0x00e90200; 64 random bytes at 0x00e90280; 16 random bytes at 0x00e90300; vtable slot +0x94 (0 args) stubbed; the stubbed vtable laid at `c4+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x18`, or `ecx+0x18` when the word at `+0x14` is set; a circular list in `c0..c3` at 0xe90000+0x10k, next `+0x4`, prev `+0x0`, head the word at `m+0xdc`; the stubbed vtable laid at `c5+0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; the stubbed vtable laid at `c7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_modify_group_order") else {
        return;
    };
    let obj = super::Obj {
        name: "modify_group_order",
        full: "Unit::modify_group_order@005e3530",
        cells: 9,
        calls: true,
        eax: false,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a` is the key, the two values, then slot +0x94's answer. The tail node is
        // `m+0xdc` (always set here: the original reads `[this+0xd4]` after the loop
        // whether or not it is), the nodes `c0..c3` at `NODES + 0x10 k` (`+0` prev, `+4`
        // next, `+8` the order, `+0xc` the cached byte), node k's order a window of
        // `c4..c7` at `ORDERS + 0x80 j`; slot +0x94 answers the order's own +0x18 when
        // its +0x14 is set, and a record whose +0xc (the order's +0x24) is the key has
        // its +4 and +8 (the order's +0x1c and +0x20) set to the values.
        const NODES: i32 = 0xe9_0000;
        const ORDERS: i32 = 0xe9_0100;
        let node = |addr: i32| ((addr - NODES) / 0x10) as usize;
        let order = |c: &[Vec<u8>], k: usize| 4 + ((rd(&c[k], 8) - ORDERS) / 0x80) as usize;
        let at = |m: &mut [u8], c: &[Vec<u8>], k: usize| {
            wr(m, 0xd4, NODES + 0x10 * k as i32);
            wr(m, 0xcc, rd(&c[k], 8));
            m[0xd0] = c[k][0xc];
        };
        let tail = rd(m, 0xdc);
        assert!(tail != 0, "the sweep draws a tail");
        let mut k = node(rd(&c[node(tail)], 4));
        at(m, c, k);
        loop {
            let o = order(c, k);
            log.push(format!("s0:c{o}+0x0"));
            if rd(&c[o], 0x14) != 0 && rd(&c[o], 0x24) == a[0] {
                wr(&mut c[o], 0x1c, a[1]);
                wr(&mut c[o], 0x20, a[2]);
            }
            k = node(rd(&c[k], 4));
            at(m, c, k);
            if rd(&c[k], 0) == rd(m, 0xdc) {
                break;
            }
        }
        0
    });
}
