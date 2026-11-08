//! `Armies::emergency@006f3250` under the emulator against the port (item 2852; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::emergency@006f3250`, run by `tools/emu/sweep_obj.py` on its row `armies_emergency` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), callee 0x006f93d0 (1 args) stubbed; the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `p6+8` points at 64 random bytes less `0` (placed); the word at `p6+12` points at 64 random bytes less `0` (placed); the word at `p7+0` points at 64 random bytes less `0` (placed); the word at `p7+4` points at 64 random bytes less `0` (placed); the word at `p7+8` points at 64 random bytes less `0` (placed); the word at `p7+12` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x8 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc09700`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_emergency") else {
        return;
    };
    let obj = super::Obj {
        name: "emergency",
        full: "Armies::emergency@006f3250",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Owner `a[0]`'s leader (`p[a[0]]`, two flag words) is live (bit 0), not a type with `& 0xc == 4`, and has none
        // of `& 0xa` in the second word: each of its armies (the count `p[2 + 2·who]`, the army records
        // `p[8 + 4·who + i]`) that is occupied (the short at +0 not zero) has its words at +0x30 and +0x34
        // set to -1 and `Army::process` told 1 (item 2852).
        let who = a[0] as usize;
        let (f0, f1) = (rd(&p[who], 0), rd(&p[who], 4));
        if f0 & 1 != 0 && f0 & 0xc != 4 && f1 & 0xa == 0 {
            for i in 0..rd(&p[2 + 2 * who], 0).max(0) as usize {
                let k = 8 + 4 * who + i;
                if p[k][0] != 0 || p[k][1] != 0 {
                    wr(&mut p[k], 0x30, -1);
                    wr(&mut p[k], 0x34, -1);
                    log.push(format!("s0:p{k}+0x0,1"));
                }
            }
        }
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
