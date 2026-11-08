//! `Build::find_target@00622c80` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Build::find_target@00622c80`, run by `tools/emu/sweep_obj.py` on its row `build_find_target` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 144 random bytes of object, 0 argument(s), vtable slot +0x130 (0 args) stubbed; callee 0x00648da0 (5 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); stub 0 answers 0..40 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 writes a value in 0..255 through its argument 1, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_find_target") else {
        return;
    };
    let obj = super::Obj {
        name: "find_target",
        full: "Build::find_target@00622c80",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The search radius is the larger of the type record's two ranges (`+0x234`, `+0x238`)
            // plus twice the slot's answer, in 0x60 units; the found target's low byte, written
            // through the out-pointer, lands at +0x81, the callee's answer is kept as a short
            // at +0x7c, and bit 2 of the short at +0x60 is cleared.
            let reach = rd(&p[0], 0x234).max(rd(&p[0], 0x238));
            let range = reach.wrapping_add(a[0].wrapping_mul(2)).wrapping_mul(0x60);
            log.push(format!("m+0x0,{range},sp-0xc,1,0,0"));
            let flags = u16::from_le_bytes([m[0x60], m[0x61]]) & 0xfffb;
            m[0x60..0x62].copy_from_slice(&flags.to_le_bytes());
            m[0x7c..0x7e].copy_from_slice(&(a[1] as i16).to_le_bytes());
            m[0x81] = a[2] as u8;
            0
        }
    });
}
