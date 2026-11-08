//! `MapFairness::enable_team_mode@0068a370` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MapFairness::enable_team_mode@0068a370`, run by `tools/emu/sweep_obj.py` on its row `map_fairness_enable_team_mode` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 120 random bytes of object, 1 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_fairness_enable_team_mode") else {
        return;
    };
    let obj = super::Obj {
        name: "enable_team_mode",
        full: "MapFairness::enable_team_mode@0068a370",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = rd(m, 0x74);
            if a[0] < n {
                let per = (n / a[0]).max((n + 1) / a[0]);
                for i in 0..n {
                    wr(m, 0x54 + 4 * i as usize, i / per);
                }
            }
            0
        }
    });
}
