//! `Armies::diplo_change@006f30f0` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::diplo_change@006f30f0`, run by `tools/emu/sweep_obj.py` on its row `armies_diplo_change` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), 224 random bytes at 0x00c09700; callee 0x006f93d0 (1 args) stubbed; the word at `c0+0x10+a[0]*0x1c` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 4 random bytes less `0` (placed); the word at `p8+4` points at 4 random bytes less `0` (placed); the word at `p8+8` points at 4 random bytes less `0` (placed); the word at `p8+0xc` points at 4 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x8 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_diplo_change") else {
        return;
    };
    let obj = super::Obj {
        name: "diplo_change",
        full: "Armies::diplo_change@006f30f0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Leader a0 (the image's leaders, 0x6eec a row) alive (+0 bit 1), its
            // bits 0xc not 4, and +4 clear of 0xa: each live army of its (the
            // armies table at 0xc09700: the count +4, the list +0x10) processed (1).
            let r = &p[a[0] as usize];
            let w = rd(r, 0);
            let mut calls = Vec::new();
            if w & 1 != 0 && w & 0xc != 4 && r[4] & 0xa == 0 {
                let n = rd(&c[0], 4 + 0x1c * a[0] as usize);
                for k in 0..n.max(0) as usize {
                    if p[9 + k][0] != 0 || p[9 + k][1] != 0 {
                        calls.push(format!("p{}+0x0,1", 9 + k));
                    }
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        }
    });
}
