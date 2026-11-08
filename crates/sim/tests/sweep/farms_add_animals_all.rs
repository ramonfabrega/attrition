//! `Farms::add_animals@008d92d0` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Farms::add_animals@008d92d0`, run by `tools/emu/sweep_obj.py` on its row `farms_add_animals_all` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 16 random bytes at 0x00c0a908; callee 0x008d8f30 (1 args) stubbed; the word at `c0+0xc` points at 1536 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("farms_add_animals_all") else {
        return;
    };
    let obj = super::Obj {
        name: "add_animals",
        full: "Farms::add_animals@008d92d0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The farms table (count at 0xc0a908, rows at *0xc0a914, 0xc0 apiece): each row whose byte +0xbc is set is handed to the per-pasture overload, ecx the count.
            let n = rd(&c[0], 0);
            let mut calls = Vec::new();
            for i in 0..n.max(0) as usize {
                if p[0][i * 0xc0 + 0xbc] != 0 {
                    calls.push(format!("{n},{i}"));
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
