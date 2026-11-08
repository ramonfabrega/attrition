//! `ObjectData::attack_dist@006488f0` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::attack_dist@006488f0`, run by `tools/emu/sweep_obj.py` on its row `object_data_attack_dist_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 4 argument(s), the global cell 0x00cae5fc points at 4096 random bytes; vtable slot +0x1c (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `0xc0ab84+0x1c*a[1]` points at 4 random bytes less `4*a[0]` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*u8(m,9)` points at 4 random bytes less `4*i16(m,0xa)` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 488 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 4 random bytes less `4*a[0]` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p6+0x18` points at 580 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*u8(m,9)` points at 4 random bytes less `4*i16(m,0xa)` (placed); the word at `p8+0` points at 32 random bytes less `0` (placed); the word at `p9+0x18` points at 580 random bytes less `0` (placed); the word at `m+0x18` points at 572 random bytes less `0` (placed); stub 2's answer points at 32 random bytes less `0` (placed); the word at `p12+0x18` points at 572 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_attack_dist_at") else {
        return;
    };
    let obj = super::Obj {
        name: "attack_dist",
        full: "ObjectData::attack_dist@006488f0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` `o`, `a[1]` `who` (the target), `a[2]`/`a[3]` the point
        // asked from; the stubs' answers from `a[4]`: `+0xc0` 5 (the
        // attacker's plane test), `+0x40` 6 (the target building's
        // record, `p[12]`); `+0x1c` answers `a[7]` (the target) then
        // `a[8]` (`this`). The target's object is `p[1]` (its point at
        // `+0x10`/`+0x14`, each XOR 0x63637), the attacker's `p[3]` with
        // its type `p[4]` (`+0x1e4`); the units' types `p[7]` (target)
        // and `p[10]` (attacker), the buildings' `p[13]` (target) and
        // `p[11]` (`this + 0x18`). `c[0]` is `div_3_table`, filled.
        use crate::combat::{Profile, attack_dist, extent};
        use crate::world::Pos;
        let unit = |t: &[u8]| Profile {
            block_radius: rd(t, 0x240),
            ..Profile::default()
        };
        let building = |t: &[u8]| Profile {
            x_size: rd(t, 0x234),
            y_size: rd(t, 0x238),
            ..Profile::default()
        };
        let at = Pos { x: a[2], y: a[3] };
        let target = Pos {
            x: rd(&p[1], 0x10) ^ 0x63637,
            y: rd(&p[1], 0x14) ^ 0x63637,
        };
        let plane = a[5] != 0 && rd(&p[4], 0x1e4) & 0x800_0000 == 0;
        let (t_b, a_b) = (a[7] != 0, a[8] != 0);
        let t_ext = extent(&if t_b { building(&p[13]) } else { unit(&p[7]) }, t_b);
        let a_ext = extent(&if a_b { building(&p[11]) } else { unit(&p[10]) }, a_b);
        i64::from(attack_dist(at, target, a_ext, t_ext, plane))
    });
}
