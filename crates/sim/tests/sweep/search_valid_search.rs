//! `Search::valid_search@0067daa0` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Search::valid_search@0067daa0`, run by `tools/emu/sweep_obj.py` on its row `search_valid_search` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p3 sparse table 0 (4 rows × 1 slice(s), row-major); callee 0x006edb50 (1 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 4 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x1 (`p` windows after the pointed buffers, before the places); `ecx` an integer in 0..3, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("search_valid_search") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_search",
        full: "Search::valid_search@0067daa0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `this` (`a[4]`) a player index, `a[0]` the mode, `a[1]` the other player; the
        // answers `a[2]` of `is_ally` (0x6edb50) and `a[3]` of `is_enemy` (0x6ebaa0), each
        // handed the leader row `p{this}` and the other player. A negative other: valid.
        let (mode, other, this) = (a[0], a[1], a[4]);
        let mut ally: Vec<String> = Vec::new();
        let mut enemy: Vec<String> = Vec::new();
        let mut is_ally = || {
            ally.push(format!("p{this}+0x0,{other}"));
            a[2] != 0
        };
        let mut is_enemy = || {
            enemy.push(format!("p{this}+0x0,{other}"));
            a[3] != 0
        };
        let ok = if other < 0 {
            true
        } else {
            match mode {
                1 => other == this,
                2 => !(is_ally() || is_enemy()),
                3 => is_enemy(),
                4 => is_ally(),
                5 => !is_ally(),
                6 => other != this,
                7 => !is_enemy(),
                _ => true,
            }
        };
        let tok = |v: &Vec<String>| {
            if v.is_empty() {
                "-".to_string()
            } else {
                v.join("/")
            }
        };
        log.push(tok(&ally));
        log.push(tok(&enemy));
        i64::from(ok)
    });
}
