//! `Animal::process@005d72c0` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Animal::process@005d72c0`, run by `tools/emu/sweep_obj.py` on its row `animal_process` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 0 argument(s), vtable slot +0x188 (0 args) stubbed; callee 0x005e0230 (0 args) stubbed; the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xf4` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 4 random bytes less `0` (placed); the word at `p1+4` points at 4 random bytes less `0` (placed); the word at `p1+8` points at 4 random bytes less `0` (placed); the word at `p1+12` points at 4 random bytes less `0` (placed); the word at `p1+16` points at 4 random bytes less `0` (placed); the word at `p1+20` points at 4 random bytes less `0` (placed); the word at `p1+24` points at 4 random bytes less `0` (placed); the word at `p1+28` points at 4 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("animal_process") else {
        return;
    };
    let obj = super::Obj {
        name: "process",
        full: "Animal::process@005d72c0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `p0` the type (`+0x304`), `p1` the figure array, `p2..`
            // the figures it points at: figure k is window `p(2+k)`.
            let mut cx = (m, &*p, &mut *log);
            crate::units::anim::animal_process_steps(
                &mut cx,
                |(_, _, log)| log.push("s0:m+0x0".into()),
                |(m, p, _)| (m[0xb5] as i8, rd(&p[0], 0x304), rd(m, 0xe8)),
                |(_, _, log), walk| {
                    for &k in walk {
                        log.push(format!("s1:p{}+0x0", 2 + k));
                    }
                },
            );
            0
        }
    });
}
