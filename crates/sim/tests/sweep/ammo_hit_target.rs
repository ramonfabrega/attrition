//! `Ammo::hit_target@00678f90` under the emulator against the port (item 3200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Ammo::hit_target@00678f90`, run by `tools/emu/sweep_obj.py` on its row `ammo_hit_target` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; vtable slot +0xbc (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; the word at `c0+0x14` points at 64 random bytes less `0` (placed); the word at `c1+0` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p1+0x18` points at 784 random bytes less `0` (placed); stub 2's answer points at 64 random bytes less `0` (placed); the word at `p4+0x18` points at 576 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [0, 0, 1] (90%); the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("ammo_hit_target") else {
        return;
    };
    let obj = super::Obj {
        name: "hit_target",
        full: "Ammo::hit_target@00678f90",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 1000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // m `this` (the ammo); p2 the object the ammo found (`+8` flags, `+0x10`/`+0x14` its
        // position, xor 0x63637), p3 the found unit's type (`+0x300`), p5 the record slot +0xb0
        // answers (`+0x234`/`+0x238` the reach). a[0] slot +0xbc, a[1] slot +0x18 (is a unit).
        let rdw =
            |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
        let wrw = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
        if rdw(m, 0x4c) < 0 || rdw(m, 0x48) < 0 || rdw(&p[2], 8) & 1 == 0 || a[0] == 0 {
            0
        } else {
            let (ox, oy) = (rdw(&p[2], 0x10) ^ 0x63637, rdw(&p[2], 0x14) ^ 0x63637);
            let mut d = crate::world::vector_dist(
                ox.wrapping_sub(rdw(m, 0x18)).wrapping_abs(),
                oy.wrapping_sub(rdw(m, 0x1c)).wrapping_abs(),
            );
            if i16::from_le_bytes([m[6], m[7]]) > 100 {
                d = d.wrapping_sub(d >> 31) >> 1;
            }
            let (kx, ky) = (
                rdw(&p[5], 0x234).wrapping_mul(0x60),
                rdw(&p[5], 0x238).wrapping_mul(0x60),
            );
            let hit = if a[1] == 0 {
                rdw(m, 0x18) <= kx.wrapping_add(ox)
                    && ox.wrapping_sub(kx) <= rdw(m, 0x18)
                    && rdw(m, 0x1c) <= ky.wrapping_add(oy)
                    && oy.wrapping_sub(ky) <= rdw(m, 0x1c)
            } else {
                d <= rdw(&p[3], 0x300)
            };
            if hit {
                1
            } else {
                wrw(m, 0x4c, -1);
                wrw(m, 0x48, -1);
                0
            }
        }
    });
}
