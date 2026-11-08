//! `City::count_gather_slots@00737dc0` under the emulator against the port (item 3219; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `City::count_gather_slots@00737dc0`, run by `tools/emu/sweep_obj.py` on its row `city_count_gather_slots` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p15 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; argument 0 names 24 random bytes (null a third of the rows); argument 1 names 24 random bytes (null a third of the rows); vtable slot +0xac (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; callee 0x00630450 (2 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p2+8` points at 32 random bytes less `0` (placed); the word at `p2+12` points at 32 random bytes less `0` (placed); the word at `c1+0` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 144 random bytes less `0` (placed); the word at `p7+4` points at 144 random bytes less `0` (placed); the word at `p7+8` points at 144 random bytes less `0` (placed); the word at `p7+12` points at 144 random bytes less `0` (placed); the word at `p8+0x18` points at 32 random bytes less `0` (placed); the word at `p9+0x18` points at 32 random bytes less `0` (placed); the word at `p10+0x18` points at 32 random bytes less `0` (placed); the word at `p11+0x18` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 0 answers `p8+j`, the data of the object its `ecx` is in `p3+j` (j < 4), linked at `+0x74`; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 2 answers -2..8 every row; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_count_gather_slots") else {
        return;
    };
    let obj = super::Obj {
        name: "count_gather_slots",
        full: "City::count_gather_slots@00737dc0",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a[0]`/`a[1]` the out arrays (null when 0; windows p0/p1,
            // six words each), stub 1 (`vtable +0x90`, is a gatherer)
            // answers `a[5..9]`, stub 2 (`BuildData::num_gatherers`)
            // `a[9..13]`, each in call order. The chain: head short
            // `+8`, record `p[8 + i]`'s next short `+0x74`, its type
            // window `p[12 + i]` with the ident at `+4`, `gather_max`
            // the signed byte `+0x80`.
            let (mut ga, mut ng) = (0usize, 0usize);
            let (o1, o2) = (a[0] != 0, a[1] != 0);
            let mut out1 = [0i32; 6];
            let mut out2 = [0i32; 6];
            let total = crate::ai_types::gather_slots_walk(
                i32::from(i16::from_le_bytes([m[8], m[9]])),
                |i| i32::from(i16::from_le_bytes([p[8 + i][0x74], p[8 + i][0x75]])),
                |_| {
                    ga += 1;
                    a[5 + (ga - 1).min(3)] != 0
                },
                |i| i32::from_le_bytes([p[12 + i][4], p[12 + i][5], p[12 + i][6], p[12 + i][7]]),
                |i| i32::from(p[8 + i][0x80] as i8),
                |_| {
                    ng += 1;
                    a[9 + (ng - 1).min(3)]
                },
                o1.then_some(&mut out1),
                o2.then_some(&mut out2),
            );
            for (k, w) in [(0usize, &out1), (1, &out2)] {
                if [o1, o2][k] {
                    for (j, v) in w.iter().enumerate() {
                        p[k][4 * j..4 * j + 4].copy_from_slice(&v.to_le_bytes());
                    }
                }
            }
            i64::from(total)
        }
    });
}
