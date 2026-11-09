//! `WorldData::was_seen@006b53f0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::was_seen@006b53f0`, run by `tools/emu/sweep_obj.py` on its row `world_data_was_seen` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 368 random bytes of object, 3 argument(s), the global cell 0x00c061e8 points at 64 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006edb50 (1 args) stubbed; the word at `m+0x134` points at 448 random bytes less `0` (placed); the word at `m+0x160` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x125e:0x10, +0x12de:0x10, +0x59e4:0x4, +0x6929:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_was_seen") else {
        return;
    };
    let obj = super::Obj {
        name: "was_seen",
        full: "WorldData::was_seen@006b53f0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let lead = a[2];
            let b = 2 + 5 * lead.clamp(0, 7) as usize;
            let (px, py) = (rd(&p[0], 0), rd(&p[1], 0));
            let flags = rd(&p[b], 0);
            let sh = |w: &[u8], i: usize| i16::from_le_bytes([w[2 * i], w[2 * i + 1]]);
            let mut call: Option<String> = None;
            let r = if lead >= 8
                || rd(&c[0], 0x30) as u8 > 1
                || flags & 0x1000 != 0
                || flags & 0x800 != 0
                || sh(&p[b + 3], 0) != 0
            {
                1
            } else {
                let xs = rd(m, 0);
                let idx = (py >> 1).wrapping_mul(xs).wrapping_add(px >> 1) as usize;
                let ent = &p[42][28 * idx..28 * idx + 28];
                let who = ent[0xf] as i8 as i32;
                let region = i16::from_le_bytes([ent[4], ent[5]]) as usize;
                let mut hit = false;
                if who >= 0 {
                    call = Some(format!("r+0x0,{who}"));
                    if a[3] != 0 {
                        let wb = 2 + 5 * who as usize;
                        hit = sh(&p[wb + 1], region) != 0 || sh(&p[wb + 2], region) != 0;
                    }
                }
                if hit {
                    1
                } else {
                    let fog = rd(m, 0xc);
                    i64::from(p[43][fog.wrapping_mul(py).wrapping_add(px) as usize] & p[b + 4][0])
                }
            };
            for k in 0..8 {
                log.push(match &call {
                    Some(t) if k == lead => t.clone(),
                    _ => "-".into(),
                });
            }
            log.push("-".into());
            r
        }
    });
}
