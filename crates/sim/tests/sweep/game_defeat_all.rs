//! `Game::defeat_all@00592aa0` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::defeat_all@00592aa0`, run by `tools/emu/sweep_obj.py` on its row `game_defeat_all` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c0620c points at 560 random bytes; callee 0x006ecb00 (3 args) stubbed; callee 0x006ec560 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_defeat_all") else {
        return;
    };
    let obj = super::Obj {
        name: "defeat_all",
        full: "Game::defeat_all@00592aa0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The scene's borders flagged for a recalc (+0x22a), then the leaders
            // table walked end to end: a live row (bit 1) whose bits 0x42 read 2 is
            // marked 0x80 and defeated (8, -1, 0); every live row's score is
            // computed (1). Each stub's calls are split by the row.
            c[0][0x22a] = 1;
            let mut defeat = Vec::new();
            let mut score = Vec::new();
            for row in p.iter_mut().take(8) {
                let w = rd(row, 0);
                let (mut d, mut s) = ("-".to_string(), "-".to_string());
                if w & 1 != 0 {
                    if w & 0x42 == 2 {
                        wr(row, 0, w | 0x80);
                        d = "r+0x0,8,-1,0".into();
                    }
                    s = "r+0x0,1".into();
                }
                defeat.push(d);
                score.push(s);
            }
            defeat.push("-".into());
            score.push("-".into());
            log.extend(defeat);
            log.extend(score);
            0
        }
    });
}
