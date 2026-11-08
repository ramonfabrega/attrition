//! `Game::get_time_limit@00593f80` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::get_time_limit@00593f80`, run by `tools/emu/sweep_obj.py` on its row `game_get_time_limit` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00e80338 points at 768 random bytes; 4 random bytes at 0x00cc21b4; 4 random bytes at 0x00c8cd00; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cc30 (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in [0, 4096], not the object, the row's last; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_get_time_limit") else {
        return;
    };
    let obj = super::Obj {
        name: "get_time_limit",
        full: "Game::get_time_limit@00593f80",
        cells: 4,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let ecx = a[3];
            let mut l: [String; 3] = std::array::from_fn(|_| "-".to_string());
            let sel = c[0][0x3c] as usize;
            if sel < 8 {
                if ecx != 0 {
                    l[0] = format!("{ecx},c1+0x{:x}", 0x58 * sel);
                }
                log.extend(l);
                i64::from(rd(&c[1], 0x58 * sel + 0x3c))
            } else if c[0][0x821] & 0x10 == 0 && c[0][0x822] & 2 == 0 {
                log.extend(l);
                -1
            } else {
                let custom = rd(&c[2], 0);
                if ecx != 0 {
                    let base = rd(&c[3], 0);
                    l[1] = format!("{ecx},{custom}");
                    l[2] = format!(
                        "{ecx},{}/{ecx},{}",
                        base.wrapping_add(0x5d5c),
                        base.wrapping_add(0x5d70)
                    );
                }
                log.extend(l);
                i64::from(if custom == 0 { 300 } else { custom })
            }
        }
    });
}
