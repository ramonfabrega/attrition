//! `World::set_oil_at@006b2a10` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_oil_at@006b2a10`, run by `tools/emu/sweep_obj.py` on its row `world_set_oil_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 312 random bytes; the global cell 0x00c0618c points at 344 random bytes; the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c0a0f0 points at 16 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; vtable slot +0x90 (0 args) stubbed; callee 0x00653f30 (3 args) stubbed; the word at `c0+0x134` points at 448 random bytes less `0` (placed); the word at `c3+0` points at 32 random bytes less `0` (placed); the word at `c3+4` points at 32 random bytes less `0` (placed); the word at `c3+8` points at 32 random bytes less `0` (placed); the word at `c3+12` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_oil_at") else {
        return;
    };
    let obj = super::Obj {
        name: "set_oil_at",
        full: "World::set_oil_at@006b2a10",
        cells: 4,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Every good of the world's list (`c[1]+0x154` of them, the records `p[3 + g]`, each with its type record `p[7 + g]`)
        // that is live (bit 0 of +8), of type 5 (the type word at +4) and sits at the tile (`p[0]`, `p[1]`)
        // — its words at +0x10 and +0x14 xor 0x63637, shifted down 8, index the table `c[2]` — is told to
        // go (slot +0x90); then the tile's flags (`p[2]`, 0x1c bytes a tile, `c[0]` wide) lose 0x800, or, when
        // `a[2]` is set, gain it and a type-5 good is made at the tile's centre (item 2852).
        let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
        let tile = ((y.wrapping_mul(rd(&c[0], 0)).wrapping_add(x)) as usize) * 0x1c;
        for g in 0..rd(&c[1], 0x154).max(0) as usize {
            let rec = &p[3 + g];
            let at = |off: usize| rd(&c[2], 4 * ((rd(rec, off) ^ 0x63637) >> 8) as usize);
            if rec[8] & 1 != 0 && rd(&p[7 + g], 4) == 5 && x == at(0x10) && y == at(0x14) {
                log.push(format!("s0:p{}+0x0", 3 + g));
            }
        }
        let flags = u16::from_le_bytes([p[2][tile], p[2][tile + 1]]);
        if a[2] == 0 {
            p[2][tile..tile + 2].copy_from_slice(&(flags & 0xf7ff).to_le_bytes());
        } else {
            p[2][tile..tile + 2].copy_from_slice(&(flags | 0x800).to_le_bytes());
            let (cx, cy) = (x * 0x300 + 0x180, y * 0x300 + 0x180);
            log.push(format!("s1:{cy},5,{cx},{cy}"));
        }
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
