//! `Unit::do_group_attack_to@005e74e0` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_group_attack_to@005e74e0`, run by `tools/emu/sweep_obj.py` on its row `unit_do_group_attack_to` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; callee 0x005e79a0 (1 args) stubbed; vtable slot +0xcc (0 args) stubbed; callee 0x005ff9c0 (5 args) stubbed; callee 0x005f22a0 (1 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); argument 0 copies `p1+0x8` (45%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_group_attack_to") else {
        return;
    };
    let obj = super::Obj {
        name: "do_group_attack_to",
        full: "Unit::do_group_attack_to@005e74e0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` the order; the stubs' answers `a[1..5]` (do_group_move, slot 0xcc, find_melee_target,
        // do_attack_to_pause), `a[5]` 1 when the slot is the image's `is_supply`. `m+0xdc` the order list (`p[0]`,
        // null a third of the rows), whose node `p[1]` (the word at `p[0]+4`) gives `m+0xd4` its address, `m+0xcc` its
        // `+8` and `m+0xd0` its `+0xc`; the current order is `m+0xcc`, or 0 for a null list. When it is the
        // argument and the unit's id plus the frame (`c[0]+0x550`) is a multiple of 15: a type (`p[2]`) with
        // `+0x1e8` clear pauses, otherwise `is_supply` (bit 0x40 of its `+0x2b8`, or the slot's answer) clear
        // finds a melee target and set pauses.
        log.push(format!("s0:m+0x0,{}", a[0]));
        let cur = if rd(m, 0xdc) == 0 {
            0
        } else {
            let node = rd(&p[0], 4);
            wr(m, 0xd4, node);
            let w8 = rd(&p[1], 8);
            wr(m, 0xcc, w8);
            m[0xd0] = p[1][0xc];
            w8
        };
        let id = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;
        if cur == a[0] && (id + rd(&c[0], 0x550)) % 15 == 0 {
            if rd(&p[2], 0x1e8) != 0 {
                let supply = if a[5] == 1 {
                    rd(&p[2], 0x2b8) & 0x40
                } else {
                    log.push("s1:m+0x0".to_string());
                    a[2]
                };
                if supply == 0 {
                    log.push("s2:m+0x0,-1,0,0,1,0".to_string());
                } else {
                    log.push(format!("s3:m+0x0,{}", a[0]));
                }
            } else {
                log.push(format!("s3:m+0x0,{}", a[0]));
            }
        }
        0
    });
}
