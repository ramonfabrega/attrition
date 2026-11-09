//! `Object::update_seen@00651b80` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Object::update_seen@00651b80`, run by `tools/emu/sweep_obj.py` on its row `object_update_seen_wide` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 1 argument(s), `p` windows in order: p0..p4 the place(s), in flag order, p5 stub 2's answer window; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00c0618c points at 80 random bytes; 56 random bytes at 0x00c0aec0; vtable slot +0x2c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x128 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; callee 0x006b3c60 (4 args) stubbed; callee 0x006b3d30 (4 args) stubbed; callee 0x006b4bb0 (4 args) stubbed; callee 0x0055e0ac (3 args) stubbed; the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `c2+0x14` points at 8 random bytes less `0` (placed); the word at `c2+0x30` points at 8 random bytes less `0` (placed); the word at `c3+0x0` points at 8 random bytes less `0` (placed); the word at `c3+0x1c` points at 8 random bytes less `0` (placed); stub 2 answers the address of 32 random bytes, a `p` window after the places; the word at `p5+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p1+0` laid as the address of `m` every row; the word at `p1+4` laid as the address of `m` every row; the word at `p2+0` laid as the address of `m` every row; the word at `p2+4` laid as the address of `m` every row; the word at `p3+0` laid as the address of `m` every row; the word at `p3+4` laid as the address of `m` every row; the word at `p4+0` laid as the address of `m` every row; the word at `p4+4` laid as the address of `m` every row; stub 3 answers 110..150 every row; stub 4 answers 1..1 every row; stub 6 answers 0..0 every row; stub 7 answers 1..1 every row; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 7 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 8 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 8 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 9 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 9 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 10 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); `0x00681920` run once before the rows (a table builder the startup would have run); `0x00a46980` run once before the rows (a table builder the startup would have run); stub 10 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_update_seen_wide") else {
        return;
    };
    let obj = super::Obj {
        name: "update_seen",
        full: "Object::update_seen@00651b80",
        cells: 4,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        super::object_update_seen::port(a, m, c, p, &mut *log)
    });
}
