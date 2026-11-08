//! `BuildQueue::un_queue@00630ad0` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildQueue::un_queue@00630ad0`, run by `tools/emu/sweep_obj.py` on its row `build_queue_un_queue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), callee 0x0055e0ac (3 args) stubbed; the word at `m+4` points at 160 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_queue_un_queue") else {
        return;
    };
    let obj = super::Obj {
        name: "un_queue",
        full: "BuildQueue::un_queue@00630ad0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Entry a0 below the count (+0, signed) → the entries after it (0x14 bytes each, the
            // list at +4) shift down one, up to entry a1: `memcpy(e[a0], e[a0+1], (a1-a0)*0x14 - 0x14)`
            // (stub 0, an import, recorded rather than run; `ecx` still holds the destination the
            // `lea` built, so the record leads with it).
            if a[0] < rd(m, 0) {
                let at = a[0] * 0x14;
                log.push(format!(
                    "p0+{at:#x},p0+{at:#x},p0+{:#x},{}",
                    at + 0x14,
                    (a[1] - a[0]) * 0x14 - 0x14
                ));
            } else {
                log.push("-".into());
            }
            0
        }
    });
}
