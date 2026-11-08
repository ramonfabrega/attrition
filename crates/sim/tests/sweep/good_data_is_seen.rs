//! `GoodData::is_seen@0066e4c0` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GoodData::is_seen@0066e4c0`, run by `tools/emu/sweep_obj.py` on its row `good_data_is_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 48 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x006e1370 (1 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x006b54f0 (3 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); 4 rows 0x6eec apart at `0xc061e0`, slices +0x6929:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("good_data_is_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "is_seen",
        full: "GoodData::is_seen@0066e4c0",
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
            let d = |v: i32| rd(&c[1], (((v ^ 0x63637) >> 7) as usize) * 4);
            let h = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            let (x, y) = (rd(m, 0x10), rd(m, 0x14));
            let kind = rd(&p[4], 4);
            // 1 the world's `was_seen`, 2 its `was_really_seen`
            let eax: i64 = if ever & mask != 0 {
                1
            } else {
                let mut seen = 2usize;
                if kind != 6 && kind != 0x1f && player >= 0 && (kind == 5 || a[5] != 0 || a[6] != 0)
                {
                    seen = 1;
                }
                l[seen - 1] = format!("c0+0x0,sp-0x14,sp-0x8,{player}");
                l[2 * seen] = h(d(x));
                l[2 * seen + 1] = h(d(y));
                i64::from(a[2 + seen])
            };
            log.extend(l);
            eax
        }
    });
}
