//! `BuildData::could_queue@0062da50` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::could_queue@0062da50`, run by `tools/emu/sweep_obj.py` on its row `build_data_could_queue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 140 random bytes of object, 1 argument(s), 4 random bytes at 0x00cab3a4; vtable slot +0x198 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x190 (2 args) stubbed; callee 0x00630450 (2 args) stubbed; the word at `0xe85ddc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p0+4` points at 8 random bytes less `0` (placed); the word at `p0+8` points at 8 random bytes less `0` (placed); the word at `p0+12` points at 8 random bytes less `0` (placed); the word at `p0+16` points at 8 random bytes less `0` (placed); the word at `p0+20` points at 8 random bytes less `0` (placed); the word at `p0+24` points at 8 random bytes less `0` (placed); the word at `p0+28` points at 8 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2, 3, 4, 5] (90%); stub 4 answered from [0, 1, 2, 3, 4, 5] (90%); the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_could_queue") else {
        return;
    };
    let obj = super::Obj {
        name: "could_queue",
        full: "BuildData::could_queue@0062da50",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `queue_fail` (the word at 0xcab3a4, `c[0]`) is 1 when slot +0x198 (`a[1]`) refuses the type,
        // 2 when the queue holds `queued` (`m[0x82]`) or more (`m+0x88`), or when a training
        // building (slot +0xb8, `a[2]`; laid as the base, flag `a[6]`, it asks the object at `m+0x18`'s
        // slot +0x60, `a[3]`) is handed a type whose record (`p[1 + a[0]]`) says 0x34 or 0x35 and slot
        // +0x190 (`a[4]`) and `num_gatherers` (`a[5]`) sum to 7 or more; else it stays 0 (item 2844).
        let kind = rd(&p[1 + a[0] as usize], 4);
        let (qf, r) = if a[1] == 0 {
            (1, 0)
        } else if i32::from(m[0x82]) >= rd(m, 0x88) {
            (2, 0)
        } else {
            let training = if a[6] != 0 { a[3] } else { a[2] };
            if training != 0 && (kind == 0x34 || kind == 0x35) && a[4].wrapping_add(a[5]) >= 7 {
                (2, 0)
            } else {
                (0, 1)
            }
        };
        wr(&mut c[0], 0, qf);
        r
    });
}
