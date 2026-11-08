//! `Border::process_morph@0087fd00` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Border::process_morph@0087fd00`, run by `tools/emu/sweep_obj.py` on its row `border_process_morph` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 208 random bytes of object, 0 argument(s), callee 0x00880e50 (0 args) stubbed; callee 0x004a12a0 (0 args) stubbed; the word at `m+0x4c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 80 random bytes less `0` (placed); the word at `p0+4` points at 80 random bytes less `0` (placed); the word at `p0+8` points at 80 random bytes less `0` (placed); the word at `p0+12` points at 80 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("border_process_morph") else {
        return;
    };
    let obj = super::Obj {
        name: "process_morph",
        full: "Border::process_morph@0087fd00",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The morph renders `p[0]` (a pointer word each, `m+0x40` of them): each is advanced (the answers `a[2..]` in call
        // order, 0 = still running); one that finished raises the border's rebuild flag (`m+0xc8` bit 0) when its
        // record (the window it was laid as, `p[1 + origin]`) has bit 2 at +0x4c, is pushed back to its recycler,
        // and is replaced in the array by the last pointer, which is cleared, the count falling by one; the
        // slot is not looked at again (item 2852).
        let origin: Vec<usize> = (0..4).collect();
        let mut next = 2;
        let mut i = 0;
        while (i as i32) < rd(m, 0x40) {
            let w = origin[i];
            next += 1;
            if a[next - 1] != 0 {
                if p[1 + w][0x4c] & 4 != 0 {
                    m[0xc8] |= 1;
                }
                log.push(format!("s0:p{}+0x0", 1 + w));
                log.push(format!("s1:p{}+0x0", 1 + w));
                let n = rd(m, 0x40) as usize;
                let last = rd(&p[0], 4 * (n - 1));
                wr(&mut p[0], 4 * i, last);
                wr(&mut p[0], 4 * (n - 1), 0);
                wr(m, 0x40, n as i32 - 1);
            } else {
                log.push(format!("s0:p{}+0x0", 1 + w));
            }
            i += 1;
        }
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
