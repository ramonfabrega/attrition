//! `Group::get_num@00714700` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::get_num@00714700`, run by `tools/emu/sweep_obj.py` on its row `group_get_num` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2516 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; callee 0x00711540 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_get_num") else {
        return;
    };
    let obj = super::Obj {
        name: "get_num",
        full: "Group::get_num@00714700",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The group: `+0xc` its count, `+4` a sign that sends a short group to `normalize`
        // (the stubbed direct callee), the owner byte `+0x4a`, then parallel arrays: shorts
        // at `+0x8cc`, bytes at `+0x84c`, dwords at `+0x4c`, `+0x24c`, `+0x44c`, `+0x64c`.
        // The objects table p[2 + 4 * owner + short]'s byte `+8` bit 0 says a member lives;
        // a dead one is shifted out of the four arrays and the count drops.
        log.push("-".into());
        let count = rd(m, 0xc);
        if count <= 0 {
            for o in [0xc, 0x1c, 0x18] {
                wr(m, o, 0);
            }
        } else if count < 4 && rd(m, 4) >= 0 {
            log[0] = "m+0x0".into();
        } else {
            let owner = m[0x4a] as usize;
            for i in (0..count as usize).rev() {
                let at = 0x8cc + 2 * i;
                let k = i16::from_le_bytes([m[at], m[at + 1]]) as usize;
                if p[2 + 4 * owner + k][8] & 1 != 0 {
                    continue;
                }
                let cur = rd(m, 0xc) as usize;
                for j in i..cur.saturating_sub(1) {
                    m.copy_within(0x8cc + 2 * (j + 1)..0x8cc + 2 * (j + 1) + 2, 0x8cc + 2 * j);
                    m[0x84c + j] = m[0x84c + j + 1];
                    for base in [0x4c, 0x24c, 0x44c, 0x64c] {
                        let v = rd(m, base + 4 * (j + 1));
                        wr(m, base + 4 * j, v);
                    }
                }
                wr(m, 0xc, cur as i32 - 1);
            }
        }
        i64::from(rd(m, 0xc))
    });
}
