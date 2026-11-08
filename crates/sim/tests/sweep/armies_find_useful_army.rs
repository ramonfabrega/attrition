//! `Armies::find_useful_army@006f2fe0` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Armies::find_useful_army@006f2fe0`, run by `tools/emu/sweep_obj.py` on its row `armies_find_useful_army` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 5 argument(s), 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e4127c; 4 random bytes at 0x00cb4bac; callee 0x006b52e0 (2 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); the word at `0xc09710+a[0]*0x1c` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `p1+12` points at 64 random bytes less `0` (placed); the word at `p1+16` points at 64 random bytes less `0` (placed); the word at `p1+20` points at 64 random bytes less `0` (placed); the word at `p1+24` points at 64 random bytes less `0` (placed); the word at `p1+28` points at 64 random bytes less `0` (placed); the word at `p1+32` points at 64 random bytes less `0` (placed); the word at `p1+36` points at 64 random bytes less `0` (placed); the word at `p1+40` points at 64 random bytes less `0` (placed); the word at `p1+44` points at 64 random bytes less `0` (placed); the word at `p1+48` points at 64 random bytes less `0` (placed); the word at `p1+52` points at 64 random bytes less `0` (placed); the word at `p1+56` points at 64 random bytes less `0` (placed); the word at `p1+60` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("armies_find_useful_army") else {
        return;
    };
    let obj = super::Obj {
        name: "find_useful_army",
        full: "Armies::find_useful_army@006f2fe0",
        cells: 3,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // get_tregion (stub 0, hooked) answers a[5]; vector_dist (stub 1) is the original's own code.
            // Records are p[2..18]; the leaders' flag words are c[0] and c[1], `find_dist` is c[2].
            let who = usize::try_from(a[0]).expect("a who");
            let tr = a[5];
            let leader = rd(&c[who], 0) & 0x700 != 0;
            let mut best = 99_999_999;
            let mut found = -1;
            for i in 0..16usize {
                let r = &p[2 + i];
                let up = u16::from_le_bytes([r[0], r[1]]) != 0;
                let div = rd(r, 0x14);
                if up && div != 0 && ((rd(r, 0x24) == 0 && leader) || tr == rd(r, 8)) {
                    let mut d = crate::world::vector_dist(
                        (a[1] - rd(r, 0x38)).abs(),
                        (a[2] - rd(r, 0x3c)).abs(),
                    );
                    if tr != rd(r, 8) {
                        d *= 3;
                    }
                    if d / div <= best {
                        found = i32::try_from(i).expect("an index");
                        best = d / div;
                    }
                }
            }
            wr(&mut c[2], 0, best);
            i64::from(found)
        }
    });
}
