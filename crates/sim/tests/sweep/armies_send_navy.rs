//! `Armies::send_navy@006f2c90` under the emulator against the port (item 2714; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::send_navy@006f2c90`, run by `tools/emu/sweep_obj.py` on its row `armies_send_navy` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 5 argument(s), callee 0x00680f90 (1 args) stubbed; callee 0x006f93d0 (1 args) stubbed; the word at `p5+0` points at 12 random bytes less `0` (placed); the word at `p7+0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 64 random bytes less `0` (placed); the word at `p8+4` points at 64 random bytes less `0` (placed); the word at `p8+8` points at 64 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `0xc061b8` points at 32 random bytes less `0` (placed); the word at `p16+0x10` points at 4480 random bytes less `0x2200` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x4:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc09704`, slices +0x0:0x4, +0xc:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_send_navy") else {
        return;
    };
    let obj = super::Obj {
        name: "send_navy",
        full: "Armies::send_navy@006f2c90",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let lead = a[0] as usize;
            let (fl, f2) = (rd(&p[2 * lead], 0), rd(&p[2 * lead + 1], 0));
            let mut seq_i = 0usize;
            let mut calls: Vec<String> = Vec::new();
            if fl & 5 == 1 && f2 & 0xa == 0 {
                let cnt = rd(&p[4 + 2 * lead], 0);
                for i in 0..cnt.max(0) as usize {
                    let w = 10 + 3 * lead + i;
                    let rec = &p[w];
                    if i16::from_le_bytes([rec[0], rec[1]]) == 0
                        || rd(rec, 0x24) == 0
                        || rd(rec, 0x14) < 3
                        || rd(rec, 8) < 0x40
                    {
                        continue;
                    }
                    let mut coast = |arg: i32| {
                        calls.push(format!("s0:p17+0x0,{arg}"));
                        let v = a[7 + seq_i.min(5)];
                        seq_i += 1;
                        v != 0
                    };
                    if coast(a[3]) && coast(a[4]) {
                        wr(&mut p[w], 0x30, a[1]);
                        wr(&mut p[w], 0x34, a[2]);
                        calls.push(format!("s1:p{w}+0x0,0"));
                    }
                }
            }
            log.extend(if calls.is_empty() {
                vec!["-".to_string()]
            } else {
                calls
            });
            0
        }
    });
}
