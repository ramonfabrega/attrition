//! `PathFinderData::valid_roadcoord@00688740` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinderData::valid_roadcoord@00688740`, run by `tools/emu/sweep_obj.py` on its row `path_finder_data_valid_roadcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 112 random bytes of object, 8 argument(s), `p` windows in order: p0..p14 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; vtable slot +0x3c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x64 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `m+0x68` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); the word at `c0+0x138` points at 288 random bytes less `0` (placed); the word at `p6+0x18` points at 576 random bytes less `0` (placed); the word at `p7+0x18` points at 576 random bytes less `0` (placed); the word at `p8+0x18` points at 576 random bytes less `0` (placed); the word at `p9+0x18` points at 576 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; stub 0 answers `p6+j`, the data of the object its `ecx` is in `p1+j` (j < 4), linked at `+0x1c`; stub 1 answers from its own `ecx`: the word at `+0x200`; stub 2 answers from its own `ecx`: the word at `+0x204`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_data_valid_roadcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_roadcoord",
        full: "PathFinderData::valid_roadcoord@00688740",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 4000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Windows: m the pathfinder (`can_transport` at +0x58); c0 `worldc` (+0x18 / +0x1c the tile width and
            // height, +0x138 the mask array p10); p0 the object array of player 0, p1..p4 its objects, whose
            // slot +0x3c answers the wall p6..p9 (`+0x10` / `+0x14` the position xor 0x63637, `+0x18` the type
            // p11..p14: `x_size` +0x234, `y_size` +0x238, the `is(VILLAGE)` answer +0x200, slot +0x64's +0x204).
            // `a[0..4]` the coordinates, `a[4]` / `a[6]` the two ends' object index, `a[11]` whether slot +0x64
            // was the real `is_city` (then slot +0x60's answer, else slot +0x64's own). The crate's
            // `valid_roadcoord` is driven on a `Sim` rebuilt from these windows; `covers_tile` is the original's own.
            use crate::cities::build::{BuildType, Ident};
            use crate::world::{Pos, World};
            use crate::{Sim, Tuning};
            const K: i32 = 0x63637;
            let (wt, ht) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let mut sim = Sim::new(Tuning::RON, World::new(wt / 4, ht / 4), 2);
            for y in 0..ht {
                for x in 0..wt {
                    let o = 2 * (y * wt + x) as usize;
                    let mask = u16::from_le_bytes([p[10][o], p[10][o + 1]]);
                    sim.world.set_tile_mask(Pos::new(x, y), mask);
                }
            }
            let mut end = |idx: i32| -> usize {
                let j = idx as usize;
                let ty_w = &p[11 + j];
                let city = if a[11] != 0 {
                    rd(ty_w, 0x200)
                } else {
                    rd(ty_w, 0x204)
                } != 0;
                let ty = sim.add_build_type(BuildType {
                    ident: if city { Ident::Village } else { Ident::Other },
                    x_size: rd(ty_w, 0x234),
                    y_size: rd(ty_w, 0x238),
                    ..BuildType::default()
                });
                let wall = &p[6 + j];
                let pos = Pos::new(rd(wall, 0x10) ^ K, rd(wall, 0x14) ^ K);
                let b = sim.add_building(0, pos, 1);
                sim.buildings[b].pos = pos;
                sim.buildings[b].ty = Some(ty);
                b
            };
            let ends = (end(a[4]), end(a[6]));
            i64::from(sim.valid_roadcoord(
                Pos::new(a[0], a[1]),
                Pos::new(a[2], a[3]),
                ends,
                rd(m, 0x58) != 0,
            ))
        }
    });
}
