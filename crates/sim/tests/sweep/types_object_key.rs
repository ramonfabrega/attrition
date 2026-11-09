//! `Types::object_key@00668ae0` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Types::object_key@00668ae0`, run by `tools/emu/sweep_obj.py` on its row `types_object_key` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0 argument 0; 4 random bytes at 0x00ee10a8; argument 0 names 16 random bytes; callee 0x006694a0 (2 args) stubbed; callee 0x00669330 (2 args) stubbed; stub 0 answered from [-3, 5, 7] (90%); stub 1 answered from [-3, 9, 11] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("types_object_key") else {
        return;
    };
    let obj = super::Obj {
        name: "object_key",
        full: "Types::object_key@00668ae0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` the key string (handed on with 1), `a[1]` unused, `a[2]` stub 0's answer (`unit_key`), `a[3]`
        // stub 1's (`build_key`, asked when the first is -3, the any-unit index). Both -3 reports the error --
        // skipped here by the `ignore_always` byte (`c[0]+2`, always set) -- and answers -3.
        log.push("s0:m+0x0,p0+0x0,1".to_string());
        if a[2] != -3 {
            a[2] as i64
        } else {
            log.push("s1:m+0x0,p0+0x0,1".to_string());
            a[3] as i64
        }
    });
}
