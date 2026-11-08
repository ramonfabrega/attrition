//! `City::set_name@00735be0` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `City::set_name@00735be0`, run by `tools/emu/sweep_obj.py` on its row `city_set_name` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 192 random bytes of object, 2 argument(s), callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1b6b0 (0 args) stubbed; the word at `m+0x90` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0xa4` points at 16 random bytes less `0` (placed, null a third of the rows); stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_set_name") else {
        return;
    };
    let obj = super::Obj {
        name: "set_name",
        full: "City::set_name@00735be0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let mut ops: Vec<String> = Vec::new();
            let mut hashes: Vec<String> = Vec::new();
            let mut next_hash = 4usize;
            let mut edx = "0".to_string();
            for (k, off) in [0x90usize, 0xa4].into_iter().enumerate() {
                ops.push(format!("m+0x{off:x},{edx},{}", a[k]));
                let ptr = rd(m, off);
                edx = if ptr == 0 {
                    "0".into()
                } else {
                    format!("p{k}+0x0")
                };
                if ptr == 0 {
                    wr(m, off + 0xc, 0);
                    wr(m, off + 0x10, 0);
                } else if m[off + 0xa] & 2 == 0 {
                    let len = i32::from(u16::from_le_bytes([m[off + 6], m[off + 7]]));
                    edx = format!("m+0x{:x}", off + 0x10);
                    hashes.push(format!(
                        "{},{edx}",
                        if m[off + 0xa] & 1 != 0 {
                            format!("p{k}+0x{:x}", 2 * len)
                        } else {
                            rd(&p[k], 0).wrapping_add(2 * len).to_string()
                        }
                    ));
                    m[off + 0xa] |= 2;
                    wr(m, off + 0xc, a[next_hash]);
                    next_hash += 1;
                }
            }
            log.push(ops.join("/"));
            log.push(if hashes.is_empty() {
                "-".into()
            } else {
                hashes.join("/")
            });
            0
        }
    });
}
