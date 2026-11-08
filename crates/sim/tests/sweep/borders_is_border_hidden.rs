//! `Borders::is_border_hidden@00882300` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Borders::is_border_hidden@00882300`, run by `tools/emu/sweep_obj.py` on its row `borders_is_border_hidden` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c06210 points at 672 random bytes; callee 0x006e1370 (1 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x74:0x20 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("borders_is_border_hidden") else {
        return;
    };
    let obj = super::Obj {
        name: "is_border_hidden",
        full: "Borders::is_border_hidden@00882300",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Leader a0 asked has_tribe_bonus(0x13) (its row of the leaders the
            // cell 0xc061e0 holds); with it, the console's leader (+0x298 of the
            // console) names its own index (+8): a0 not it, and not at 2 both ways
            // in the diplos (+0x74, a word an index), answers 1.
            let who = a[0] as usize;
            let mut row = vec!["-".to_string(); 9];
            row[who] = "r+0x0,19".into();
            log.extend(row);
            let mut v = 0;
            if a[1] != 0 {
                let me = rd(&c[0], 0x298) as usize;
                let mine = rd(&p[2 * me], 0);
                if a[0] != mine
                    && (rd(&p[2 * me + 1], 4 * who) != 2
                        || rd(&p[2 * who + 1], 4 * mine as usize) != 2)
                {
                    v = 1;
                }
            }
            v
        }
    });
}
