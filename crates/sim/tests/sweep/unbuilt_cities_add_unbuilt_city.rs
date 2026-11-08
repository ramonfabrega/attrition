//! `UnbuiltCities::add_unbuilt_city@0073b7c0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnbuiltCities::add_unbuilt_city@0073b7c0`, run by `tools/emu/sweep_obj.py` on its row `unbuilt_cities_add_unbuilt_city` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), 112 random bytes at 0x00e39a40; vtable slot +0x0 (1 args) stubbed; the word at `c0+0x10` points at 64 random bytes less `0` (placed); the word at `c0+0x2c` points at 64 random bytes less `0` (placed); the word at `c0+0x48` points at 64 random bytes less `0` (placed); the word at `c0+0x64` points at 64 random bytes less `0` (placed); the stubbed vtable laid at `c0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `c0+0x1c` as well as at `this`; the stubbed vtable laid at `c0+0x38` as well as at `this`; the stubbed vtable laid at `c0+0x54` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unbuilt_cities_add_unbuilt_city") else {
        return;
    };
    let obj = super::Obj {
        name: "add_unbuilt_city",
        full: "UnbuiltCities::add_unbuilt_city@0073b7c0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `UnbuiltCities::add_unbuilt_city(packed, who)`: list `who` at `0xe39a40 + 0x1c * who` (`c[0]`)
        // grows through its vtable slot 0 (`ecx` the list, handed its short at `+0xc`) when `count >=
        // cap`; then `packed` with byte 2 replaced by `who` is stored at `data[count]` (`+0x10`, the
        // place `p[who]`) and `count` goes up by one.
        let (packed, who) = (a[0], a[1] as usize);
        let base = 0x1c * who;
        let count = rd(&c[0], base + 4);
        let call = if count >= rd(&c[0], base + 8) {
            let w = i16::from_le_bytes([c[0][base + 0xc], c[0][base + 0xd]]);
            format!("c0+{base:#x},{w}")
        } else {
            "-".to_string()
        };
        log.push(call);
        let v = (packed & !0x00ff_0000) | ((who as i32 & 0xff) << 16);
        wr(&mut p[who], 4 * count as usize, v);
        wr(&mut c[0], base + 4, count + 1);
        0i64
    });
}
