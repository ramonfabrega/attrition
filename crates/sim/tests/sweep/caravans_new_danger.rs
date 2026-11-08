//! `Caravans::new_danger@0073e0c0` under the emulator against the port (item 3120; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Caravans::new_danger@0073e0c0`, run by `tools/emu/sweep_obj.py` on its row `caravans_new_danger` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 3 argument(s), `p` windows in order: p0..p31 sparse table 0 (8 rows × 4 slice(s), row-major), p32..p39 sparse table 1 (8 rows × 1 slice(s), row-major), p40..p60 the place(s), in flag order; callee 0x0073d070 (0 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xe3a2a0` points at 12 random bytes less `0` (placed); the word at `0xe3a2bc` points at 12 random bytes less `0` (placed); the word at `0xe3a2d8` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 36 random bytes less `0` (placed); the word at `p40+4` points at 36 random bytes less `0` (placed); the word at `p40+8` points at 36 random bytes less `0` (placed); the word at `p41+0` points at 36 random bytes less `0` (placed); the word at `p41+4` points at 36 random bytes less `0` (placed); the word at `p41+8` points at 36 random bytes less `0` (placed); the word at `p42+0` points at 36 random bytes less `0` (placed); the word at `p42+4` points at 36 random bytes less `0` (placed); the word at `p42+8` points at 36 random bytes less `0` (placed); the word at `p43+0x10` points at 48 random bytes less `0` (placed); the word at `p44+0x10` points at 48 random bytes less `0` (placed); the word at `p45+0x10` points at 48 random bytes less `0` (placed); the word at `p46+0x10` points at 48 random bytes less `0` (placed); the word at `p47+0x10` points at 48 random bytes less `0` (placed); the word at `p48+0x10` points at 48 random bytes less `0` (placed); the word at `p49+0x10` points at 48 random bytes less `0` (placed); the word at `p50+0x10` points at 48 random bytes less `0` (placed); the word at `p51+0x10` points at 48 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x10, +0x438:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x10 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("caravans_new_danger") else {
        return;
    };
    let obj = super::Obj {
        name: "new_danger",
        full: "Caravans::new_danger@0073e0c0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 1200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a`: 0 and 1 the coordinates, 2 `param_3` (a leader), 3 and 4 the stubs' singles (unused). Windows: p[4r..4r+3] leader r's
            // image row (flags, +8, the relations at +0x74, the caravan count at +0x438), p[32+r] its row of the `leadersc` table's
            // relations (+0x74), p[43+3k+j] leader k's caravan j, p[52+3k+j] that caravan's positions; `vector_dist` is the original's own
            // code (stub 1), `Caravan::restart_trade_route` (stub 0) is logged by its caravan's window.
            let rd = |w: &[u8], o: usize| {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            };
            let p3 = a[2] as usize;
            let mut calls: Vec<String> = Vec::new();
            for r in 0..8usize {
                if rd(&p[4 * r], 0) & 3 != 3 {
                    continue;
                }
                let who = rd(&p[4 * r + 1], 0);
                if a[2] != who
                    && !(rd(&p[4 * r + 2], 4 * p3) == 2 && rd(&p[32 + p3], 4 * who as usize) == 2)
                {
                    continue;
                }
                for j in 0..rd(&p[4 * r + 3], 0).max(0) as usize {
                    let cv = &p[43 + 3 * r + j];
                    if cv[0xc] & 3 != 3 {
                        continue;
                    }
                    let tok = || format!("p{}+0x0", 43 + 3 * r + j);
                    if rd(cv, 0x20) == 0 && rd(cv, 0x18) != 0 {
                        let pos = &p[52 + 3 * r + j];
                        for i in 0..rd(cv, 0x18).max(0) as usize {
                            let d = crate::world::vector_dist(
                                rd(pos, 16 * i).wrapping_sub(a[0]).wrapping_abs(),
                                rd(pos, 16 * i + 4).wrapping_sub(a[1]).wrapping_abs(),
                            );
                            if d < 0x601 {
                                calls.push(tok());
                                break;
                            }
                        }
                    } else {
                        calls.push(tok());
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
