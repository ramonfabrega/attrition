//! `CollCheck::collide_here@00682540` under the emulator against the port (item 3289; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CollCheck::collide_here@00682540`, run by `tools/emu/sweep_obj.py` on its row `coll_check_collide_here` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 8 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p17 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 16 random bytes at 0x00cab310 (each word in 0..3); argument 5 names 8 random bytes (null a third of the rows); argument 6 names 8 random bytes (null a third of the rows); vtable slot +0xbc (0 args) stubbed; callee 0x006820e0 (4 args) stubbed; the word at `c0+0x14` points at 8 random bytes less `0` (placed); the word at `c0+0x30` points at 8 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p8+0` points at 144 random bytes less `0` (placed); the word at `p8+4` points at 144 random bytes less `0` (placed); the word at `p9+0` points at 144 random bytes less `0` (placed); the word at `p9+4` points at 144 random bytes less `0` (placed); the word at `0xcab320` points at 108 random bytes less `0` (placed, null a third of the rows); the word at `0xcab324` points at 108 random bytes less `0` (placed, null a third of the rows); the word at `0xcab328` points at 108 random bytes less `0` (placed, null a third of the rows); the word at `0xcab32c` points at 108 random bytes less `0` (placed, null a third of the rows); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p10+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("coll_check_collide_here") else {
        return;
    };
    let obj = super::Obj {
        name: "collide_here",
        full: "CollCheck::collide_here@00682540",
        cells: 3,
        calls: true,
        eax: true,
        min_rows: 4000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a`: the unit's index, its player, the proposed cell x and y, `coll_size`, the two out
            // pointers (`p0`, `p1`'s addresses, or 0), `nocoll`; then the answers of slot +0xbc (the
            // unit's `is_on_map` when the stub is not laid), of `fill_slots` (unread) and the flag
            // that lays the image's own `UnitData::is_on_map` at slot +0xbc. c0 the objects record
            // (the player's array of objects at `+0x14 + 0x1c·player`), c1 `div_3_table`, c2 the four
            // `collcheck.get` words; p0/p1 the out cells, p4..p7 the objects and p10..p13 the units
            // (player `2·who + index`), p14..p17 the four `collcheck.locals` blocks, a null one marked
            // by its first word.
            const NULL_MARK: i32 = 0x4e49_4c4c;
            let (unit, who, x, y, size, out_x, out_y, nocoll) =
                (a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7] != 0);
            let (stub_on_map, real_on_map) = (a[8] != 0, a[10] != 0);
            let div3 = |k: i32| rd(&c[1], 4 * k as usize);
            let which = (2 * who + unit) as usize;
            // `size == 0` returns before `fill_slots` is called: no prelude and no log, and a port
            // that probed anyway (every slot read as it stands) would find bits the original never
            // looked at.
            let mut dead = [false; 4];
            if size != 0 {
                log.push(format!("m+0x0,{x},{y},{size},{}", i32::from(nocoll)));
                // The prelude: `local_30[i]` is "this slot reads as empty" — no `get`, no block, or
                // `BitMask<768>::empty`, which caches its scan in `flags`.
                for (i, d) in dead.iter_mut().enumerate() {
                    let blk = &mut p[14 + i];
                    *d = rd(&c[2], 4 * i) == 0 || rd(blk, 0) == NULL_MARK || {
                        let flags = rd(blk, 8);
                        if flags & 1 != 0 {
                            true
                        } else if flags == 0 {
                            false
                        } else {
                            let n = rd(blk, 4).clamp(0, 96) as usize;
                            if blk[0xc..0xc + n].iter().fold(0u8, |o, b| o | b) != 0 {
                                wr(blk, 8, 0);
                                false
                            } else {
                                wr(blk, 8, 1);
                                true
                            }
                        }
                    };
                }
            }
            let pos_of = |off: usize| div3((rd(&p[4 + which], off) ^ 0x6_3637) >> 4);
            let mine = crate::Pos::new(pos_of(0x10), pos_of(0x14));
            let unit_rec = &p[10 + which];
            let on_map = if real_on_map {
                (u16::from_le_bytes([unit_rec[0x82], unit_rec[0x83]]) >> 15) != 0
            } else {
                stub_on_map
            };
            // The slot a cell reads: `2·(x>>4 > (at.x - size)>>4) + (y>>4 > (at.y - size)>>4)`.
            let (lx, ly) = ((x - size) >> 4, (y - size) >> 4);
            let slot =
                |q: crate::Pos| (if q.x >> 4 <= lx { 0 } else { 2 }) + usize::from(q.y >> 4 > ly);
            let blocks: Vec<Vec<u8>> = (0..4).map(|i| p[14 + i].clone()).collect();
            let hit = crate::movement::collide::collide_here_steps(
                size,
                crate::Pos::new(x, y),
                mine,
                on_map,
                nocoll,
                |q| !dead[slot(q)],
                |q| {
                    let (rx, ry) = (q.x % 16, q.y % 16);
                    let bit = (rx * 16 + ry) as usize;
                    blocks[slot(q)][0xc + (bit >> 3)] & (1u8 << (bit & 7)) != 0
                },
            );
            let mut result = 0i64;
            if let Some(q) = hit {
                if out_x != 0 {
                    wr(&mut p[0], 0, q.x);
                }
                if out_y != 0 {
                    wr(&mut p[1], 0, q.y);
                }
                result = 1;
            }
            if log.is_empty() {
                log.push("-".into());
            }
            result
        }
    });
}
