//! `World::new_coll_block@0046d250` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::new_coll_block@0046d250`, run by `tools/emu/sweep_obj.py` on its row `world_new_coll_block` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2 the place, p3 stub 0's answer window; the global cell 0x00c06188 points at 320 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x00681fa0 (0 args) stubbed; callee 0x0055e0b2 (3 args) stubbed; the word at `c0+0x134` points at 448 random bytes less `0` (placed); stub 0 answers the address of 112 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_new_coll_block") else {
        return;
    };
    let obj = super::Obj {
        name: "new_coll_block",
        full: "World::new_coll_block@0046d250",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 1500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // A collision block (the allocator stub, answering `p3`) is initialised (+0 = 0x300, +4 = 0x60,
        // +8 = 1; the 0x60 bytes at +0xc cleared by the CRT memset, a stub here, whose call is the log),
        // then stored in the world tile (`c0` the world: width +0, tile array +0x134, 0x1c a tile) at
        // (x, y) = the words the two arguments point at, at tile offset +0x18; the tile's word is returned.
        // (The allocator's null answer — block 0 stored — is not drawn: the stub answers a window.)
        let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
        let at = ((rd(&c[0], 0) * y + x) * 0x1c + 0x18) as usize;
        wr(&mut p[3], 0, 0x300);
        wr(&mut p[3], 4, 0x60);
        wr(&mut p[3], 8, 1);
        wr(&mut p[2], at, a[2]);
        log.push("m+0x0".to_string());
        log.push("m+0x0,p3+0xc,0,96".to_string());
        i64::from(rd(&p[2], at))
    });
}
