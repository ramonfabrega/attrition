//! `Herds::process@00741cc0` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Herds::process@00741cc0`, run by `tools/emu/sweep_obj.py` on its row `herds_process` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 1536 random bytes; 32 random bytes at 0x00c0a244; callee 0x00741760 (0 args) stubbed; the word at `c1+0xc` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+0xc` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("herds_process") else {
        return;
    };
    let obj = super::Obj {
        name: "process",
        full: "Herds::process@00741cc0",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // On a frame divisible by 64 (the game's frame at +0x550), herd (frame / 64) % max(count, 5) of the herds table (count at 0xc0a244, the Herd pointers at *0xc0a250) is processed when it is in range and its byte +0x1a has bit 0 set (the tail call, ecx the herd).
            let frame = rd(&c[0], 0x550);
            let n = rd(&c[1], 0);
            let mut calls = Vec::new();
            if frame & 0x3f == 0 {
                let k = (frame >> 6) % n.max(5);
                if k < n && p[1 + k as usize][0x1a] & 1 != 0 {
                    calls.push(format!("p{}+0x0", 1 + k));
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
