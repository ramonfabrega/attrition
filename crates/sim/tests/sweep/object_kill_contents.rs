//! `Object::kill_contents@00645550` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Object::kill_contents@00645550`, run by `tools/emu/sweep_obj.py` on its row `object_kill_contents` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), 56 random bytes at 0x00c0aec0; vtable slot +0xbc (0 args) stubbed; callee 0x00617c10 (1 args) stubbed; callee 0x00606460 (1 args) stubbed; the word at `c0+0` points at 64 random bytes less `0` (placed); the word at `c0+0x1c` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 writes -1 (2 bytes) at `m+0x28` each call; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_kill_contents") else {
        return;
    };
    let obj = super::Obj {
        name: "kill_contents",
        full: "Object::kill_contents@00645550",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Slot +0xbc asked; nonzero, and the contents' head (+0x28, a short) not negative: the
            // head unit (the player's units table, cell 0xc0aec0 + 0x1c × the byte +0x3e) is let
            // out `(0)` and killed `(0)` until the head is negative — the kill unlinks it (here, the
            // stub writes -1 at the head).
            log.push("m+0x0".into());
            let head = i16::from_le_bytes([m[0x28], m[0x29]]);
            if a[0] != 0 && head >= 0 {
                let unit = rd(&p[m[0x3e] as usize], head as usize * 4);
                log.push(format!("{unit},0"));
                log.push(format!("{unit},0"));
                m[0x28..0x2a].copy_from_slice(&(-1i16).to_le_bytes());
            } else {
                log.push("-".into());
                log.push("-".into());
            }
            0
        }
    });
}
