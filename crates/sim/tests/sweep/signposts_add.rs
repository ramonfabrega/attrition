//! `Signposts::add@00897e20` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Signposts::add@00897e20`, run by `tools/emu/sweep_obj.py` on its row `signposts_add` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0; argument 0 names 24 random bytes; callee 0x00a1d590 (1 args) stubbed; callee 0x004a4390 (5 args) stubbed; callee 0x0047bb10 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("signposts_add") else {
        return;
    };
    let obj = super::Obj {
        name: "add",
        full: "Signposts::add@00897e20",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // String copy of the name onto the stack, `labels.add` handed that copy (five words
        // the stubbed copy leaves zeroed), then the two coordinates through the lists' `add`
        log.push("sp-0x3c,p0+0x0".into());
        log.push("15407716,0,0,0,0,0".into());
        log.push(format!("15407756,{}/15407780,{}", a[1], a[2]));
        i64::from(a[5])
    });
}
