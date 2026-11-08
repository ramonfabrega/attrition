//! `Unit::do_idle@0060dcd0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_idle@0060dcd0`, run by `tools/emu/sweep_obj.py` on its row `unit_do_idle` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p4 the place(s), in flag order; vtable slot +0xc4 (0 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x006032c0 (0 args) stubbed; callee 0x005f6e40 (0 args) stubbed; the word at `m+0xf4` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 160 random bytes less `0` (placed); the word at `0xc0aec0+u8(m,9)*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+i16(m,0xa)*4` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_idle") else {
        return;
    };
    let obj = super::Obj {
        name: "do_idle",
        full: "Unit::do_idle@0060dcd0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Clears the 0x8000 flag of +0x6c (restored at the end). When the animation
        // class of the first graphic piece's category (the image's `UnitAnimCat`,
        // 12 for entries 11..=14 — a table the executable initialises statically)
        // is that class, the unit's own answer (slot +0xc4, inlined as the +0x2b8
        // word's bit 0x20 when the slot is `UnitData::is_hero`) suppresses the default
        // animation; `set_anim(CHAR_DEFAULT, 0, 1)` otherwise.
        let flag = rd(m, 0x6c) & 0x8000 != 0;
        if flag {
            wr(m, 0x6c, rd(m, 0x6c) & !0x8000);
        }
        let cat = p[1][0x9c] as i8;
        let mut anim = true;
        if (11..=14).contains(&cat) {
            let hero_slot = a[4] != 0;
            let v = if hero_slot {
                rd(&p[4], 0x2b8) & 0x20
            } else {
                a[0]
            };
            if v != 0 {
                anim = false;
            }
        }
        if anim {
            log.push("s1:m+0x0,0,0,1".to_string());
        }
        m[0x88] = 0;
        m[0x89] = 0;
        log.push("s2:m+0x0".to_string());
        log.push("s3:m+0x0".to_string());
        if flag {
            wr(m, 0x6c, rd(m, 0x6c) | 0x8000);
        }
        0
    });
}
