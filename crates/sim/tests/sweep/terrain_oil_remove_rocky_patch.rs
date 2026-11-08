//! `TerrainOil::remove_rocky_patch@008a2ac0` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TerrainOil::remove_rocky_patch@008a2ac0`, run by `tools/emu/sweep_obj.py` on its row `terrain_oil_remove_rocky_patch` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 16 random bytes; callee 0x0055d063 (4 args) stubbed; callee 0x0055d055 (2 args) stubbed; the word at `m+0x14` points at 48 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+4` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+8` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+12` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+16` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+20` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+24` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+28` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+32` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+36` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+40` points at 64 random bytes less `-4` (placed, null a third of the rows); the word at `p0+44` points at 64 random bytes less `-4` (placed, null a third of the rows); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_oil_remove_rocky_patch") else {
        return;
    };
    let obj = super::Obj {
        name: "remove_rocky_patch",
        full: "TerrainOil::remove_rocky_patch@008a2ac0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `TerrainOil::remove_rocky_patch(x, y)`: the slot `patch_data[w * y + x]` (the pointer table at
        // `this+0x14`, `w` the first word of the `GameAccess::world` record at 0xc06188). A null slot is left
        // alone; a record pointer `q` (an `OilPatchData[]` with its count at `q-4`) is destroyed through
        // `_eh_vector_destructor_iterator_(q, 0x3c, count, OilPatchData::~OilPatchData)` (stub 0), freed with
        // `operator delete[](q-4, (count*16 - count)*4 + 4)` (stub 1, `cdecl`), and the slot zeroed.
        // `eax` is not read (`--ret none`). Entry k names window p(1+k); the second call's `ecx` is the leftover `count * 15`.
        let idx = rd(&c[0], 0).wrapping_mul(a[1]).wrapping_add(a[0]);
        assert!(
            (0..12).contains(&idx),
            "the draw keeps the slot inside the table"
        );
        let at = idx as usize * 4;
        let q = rd(&p[0], at);
        if q != 0 {
            let w = 1 + idx as usize;
            let count = rd(&p[w], 0);
            let ecx = count.wrapping_mul(16).wrapping_sub(count);
            let size = ecx.wrapping_mul(4).wrapping_add(4);
            log.push(format!("m+0x0,p{w}+0x4,60,{count},4874304"));
            log.push(format!("{ecx},p{w}+0x0,{size}"));
            wr(&mut p[0], at, 0);
        } else {
            log.push("-".into());
            log.push("-".into());
        }
        0
    });
}
