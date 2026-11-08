//! `Type::set_name@00663ae0` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Type::set_name@00663ae0`, run by `tools/emu/sweep_obj.py` on its row `type_set_name` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), argument 0 names 32 random bytes; callee 0x00a1eeb0 (1 args) stubbed; vtable slot +0xb0 (2 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; stub 1 answers the address of 32 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_set_name") else {
        return;
    };
    let obj = super::Obj {
        name: "set_name",
        full: "Type::set_name@00663ae0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Three string assignments around one virtual call, all callees stubbed:
        // `name = arg` (this+0x60), `display = name` (this+0xb0), then the slot-0xb0
        // method makes a temporary from `name` (out-pointer a local, `ecx` = this) and
        // its answer is assigned to this+0x74; the temporary is closed. Nothing of
        // `this` is written here, so every window is held unchanged.
        log.push("s0:m+0x60,p0+0x0".into());
        log.push("s0:m+0xb0,m+0x60".into());
        log.push("s1:m+0x0,sp-0x2c,m+0x60".into());
        log.push("s0:m+0x74,p1+0x0".into());
        log.push("s2:sp-0x2c".into());
        0
    });
}
