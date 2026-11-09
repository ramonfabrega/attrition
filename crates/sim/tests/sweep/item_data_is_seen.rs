//! `ItemData::is_seen@00677850` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ItemData::is_seen@00677850`, run by `tools/emu/sweep_obj.py` on its row `item_data_is_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 48 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x006e1370 (1 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x006b55c0 (3 args) stubbed; the word at `c0+0x134` points at 1536 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 4 rows 0x6eec apart at `0xc061e0`, slices +0x6929:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("item_data_is_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "is_seen",
        full: "ItemData::is_seen@00677850",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let player = a[0];
            let ever = m[0x20];
            let mask = p[player as usize][0];
            let mut l: [String; 6] = std::array::from_fn(|_| "-".to_string());
            let d = |v: i32, sh: u32| rd(&c[1], (((v ^ 0x63637) >> sh) as usize) * 4);
            let h = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            let (x, y) = (rd(m, 0x10), rd(m, 0x14));
            let mut seen_via = 2usize;
            let eax: i64 = if ever & mask != 0 {
                1
            } else {
                if player >= 0 && a[2] != 0 {
                    let idx = d(y, 8).wrapping_mul(rd(&c[0], 0)).wrapping_add(d(x, 8)) as usize;
                    if i32::from(p[4][idx.wrapping_mul(0x1c) + 0xf] as i8) == player {
                        seen_via = 1;
                    }
                }
                let k = seen_via;
                l[k - 1] = format!("c0+0x0,sp-0x14,sp-0x8,{player}");
                l[2 * k] = h(d(x, 7));
                l[2 * k + 1] = h(d(y, 7));
                i64::from(a[2 + k])
            };
            log.extend(l);
            eax
        }
    });
}
