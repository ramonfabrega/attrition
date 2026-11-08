//! `Ammo::check_hit@00678d90` under the emulator against the port (item 3200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Ammo::check_hit@00678d90`, run by `tools/emu/sweep_obj.py` on its row `ammo_check_hit` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00c06188 points at 768 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; callee 0x0065ca80 (13 args) stubbed; callee 0x0065ab40 (7 args) stubbed; callee 0x006624f0 (4 args) stubbed; the word at `c0+0x14` points at 64 random bytes less `0` (placed); the word at `c1+0` points at 64 random bytes less `0` (placed); the word at `c1+4` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p1+0x18` points at 784 random bytes less `0` (placed); the word at `p2+0x18` points at 784 random bytes less `0` (placed); the word at `p3+0x18` points at 768 random bytes less `0` (placed); the word at `c2+0x138` points at 17408 random bytes less `0` (placed); stub 1 answered from [-1, -1, -1, 0, 3, 5] (90%); stub 0 answers -1..1 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("ammo_check_hit") else {
        return;
    };
    let obj = super::Obj {
        name: "check_hit",
        full: "Ammo::check_hit@00678d90",
        cells: 4,
        calls: true,
        eax: true,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // m `this` (the ammo); c0 the objects record (`+0x1fc` the minimum, `+0x200` the last
        // search's player), c1 the units cell (two units), c2 the world, c3 the div_3_table; p3 the
        // shooter's object (the sound's `ecx`), p6 its type, p4/p5 the two found units' types, p7
        // the tile flags. a[0] the domain, a[1] the `find_unit` answer, a[2] the
        // `find_building_at` answer. Recorded: the miss sound, `p3+0x0,<sound>,<x>,<y>,<tile x>`.
        let rdw =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        let wrw = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let mut ret = 1;
        let mut calls: Vec<String> = Vec::new();
        let found = a[1];
        wrw(m, 0x4c, found);
        wrw(m, 0x48, rdw(&c[0], 0x200));
        if found >= 0 {
            let ty = if found == 0 { &p[4] } else { &p[5] };
            if rdw(ty, 0x300) < rdw(&c[0], 0x1fc) {
                wrw(m, 0x4c, -1);
                wrw(m, 0x48, -1);
            }
        }
        if rdw(m, 0x4c) < 0 {
            let (x, y) = (rdw(m, 0x18), rdw(m, 0x1c));
            if x >= 0
                && y >= 0
                && x < rdw(&c[2], 0x18).wrapping_mul(0xc0)
                && y < rdw(&c[2], 0x1c).wrapping_mul(0xc0)
            {
                let b = a[2];
                wrw(m, 0x4c, b);
                wrw(m, 0x48, rdw(&c[0], 0x200));
                if b >= 0 {
                    ret = 1;
                } else {
                    let tx = rdw(&c[3], ((x >> 6) as usize) * 4);
                    let ty = rdw(&c[3], ((y >> 6) as usize) * 4);
                    let at = (rdw(&c[2], 0x18).wrapping_mul(ty).wrapping_add(tx)) as usize * 2;
                    let sound = if p[7][at] & 0x30 == 0x20 { 6 } else { 7 };
                    calls.push(format!(
                        "p3+0x0,{sound},{},{},{tx}",
                        tx * 0xc0 + 0x60,
                        ty * 0xc0 + 0x60
                    ));
                    wrw(m, 0x4c, -1);
                    wrw(m, 0x48, -1);
                    ret = 0;
                }
            } else {
                wrw(m, 0x48, -1);
                ret = 0;
            }
        }
        log.push(if calls.is_empty() {
            "-".to_string()
        } else {
            calls.join("/")
        });
        ret
    });
}
