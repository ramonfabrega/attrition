//! `Herds::start_herds@00741d10` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Herds::start_herds@00741d10`, run by `tools/emu/sweep_obj.py` on its row `herds_start_herds` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 32 random bytes at 0x00c0a244; callee 0x007417f0 (0 args) stubbed; the word at `c0+0xc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+0xc` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("herds_start_herds") else {
        return;
    };
    let obj = super::Obj {
        name: "start_herds",
        full: "Herds::start_herds@00741d10",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The herds table (count at 0xc0a244, the Herd pointers at *0xc0a250): each herd whose byte +0x1a has bit 0 set is started (create_units, ecx the herd); 1.
            let n = rd(&c[0], 0);
            let mut calls = Vec::new();
            for i in 0..n.max(0) as usize {
                if p[1 + i][0x1a] & 1 != 0 {
                    calls.push(format!("p{}+0x0", 1 + i));
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            1
        }
    });
}
