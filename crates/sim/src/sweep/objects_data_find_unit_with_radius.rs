//! `ObjectsData::find_unit_with_radius@00659890` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `ObjectsData::find_unit_with_radius@00659890`, run by `tools/emu/sweep_objects_data_find_unit_with_radius.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 9 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p15 sparse table 1 (8 rows × 1 slice(s), row-major), p16..p23 sparse table 2 (8 rows × 1 slice(s), row-major), p24..p55 the place(s), in flag order; the global cell 0x00c0618c points at 520 random bytes; the global cell 0x00c061e8 points at 1760 random bytes; 8 random bytes at 0x00cbe330 (each word in 1000..2000); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p25+0` points at 64 random bytes less `0` (placed); the word at `p25+4` points at 64 random bytes less `0` (placed); the word at `p26+0` points at 64 random bytes less `0` (placed); the word at `p26+4` points at 64 random bytes less `0` (placed); the word at `p27+0` points at 64 random bytes less `0` (placed); the word at `p27+4` points at 64 random bytes less `0` (placed); the word at `p28+0` points at 32 random bytes less `0` (placed); the word at `p28+4` points at 32 random bytes less `0` (placed); the word at `p29+0` points at 32 random bytes less `0` (placed); the word at `p29+4` points at 32 random bytes less `0` (placed); the word at `p30+0` points at 32 random bytes less `0` (placed); the word at `p30+4` points at 32 random bytes less `0` (placed); the word at `p31+0` points at 32 random bytes less `0` (placed); the word at `p31+4` points at 32 random bytes less `0` (placed); the word at `p40+0x18` points at 584 random bytes less `0` (placed); the word at `p41+0x18` points at 584 random bytes less `0` (placed); the word at `p42+0x18` points at 584 random bytes less `0` (placed); the word at `p43+0x18` points at 584 random bytes less `0` (placed); the word at `p44+0x18` points at 584 random bytes less `0` (placed); the word at `p45+0x18` points at 584 random bytes less `0` (placed); the word at `p46+0x18` points at 584 random bytes less `0` (placed); the word at `p47+0x18` points at 584 random bytes less `0` (placed); the stubbed vtable laid at `p32+0`; stub 2 answers 8 values in call order, printed after the answers; stub 2 answers 0..1 every row; stub 0 answers from its own `ecx`: the word at `+0x20`; stub 1 answers from its own `ecx`: the word at `+0x24`; 8 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_unit_with_radius.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The port is the crate's own `Sim::find_unit_with_radius`, driven on a `Sim` built from the row's windows
                // (the array arm: `r < 0`, or the `circle_radius` draw above `total_units`). c0 the `objects` record (counts
                // at `+0x15c + 4 * who`, the nearest-so-far pair at `+0x1fc`/`+0x200`); p16..p23 the leaders' flag word
                // (bit 0: the player is in the game); p32.. the objects (is_active stub reads `+0x20`, is_on_map `+0x24`,
                // the point is `+0x10`/`+0x14` xor 0x63637), p48.. the types (`big_radius` `+0x244`). a[0], a[1] the point,
                // a[4] the radius (-1: unbounded, mapped to a huge `r_coll`), a[6] the filter, a[12..] valid_filter's answers.
                use crate::world::Pos;
                use crate::{Sim, Tuning, Unit, UnitType, World};
                let mut s = Sim::new(Tuning::RON, World::new(8, 8), 8);
                let mut exempt: Option<usize> = None;
                let mut calls = 0usize;
                let filtered = (1..=0x17).contains(&a[6]);
                for pl in 0..4usize {
                    if rd(&p[16 + pl], 0) & 1 == 0 {
                        continue;
                    }
                    let count = rd(&c[0], 0x15c + 4 * pl) as usize;
                    for j in 0..count {
                        let k = 2 * pl + j;
                        let obj = &p[32 + k];
                        let prof = &p[48 + k];
                        let t = s.add_unit_type(UnitType {
                            combat: crate::combat::Profile {
                                big_radius: rd(prof, 0x244),
                                ..Default::default()
                            },
                            ..UnitType::default()
                        });
                        let active = rd(obj, 0x20) != 0;
                        let on_map = rd(obj, 0x24) != 0;
                        let mut u = Unit::new(
                            pl as u8,
                            k as i16,
                            Pos::new(rd(obj, 0x10) ^ 0x63637, rd(obj, 0x14) ^ 0x63637),
                            i32::from(active),
                        );
                        u.ty = Some(t);
                        u.on_map = on_map;
                        let idx = s.add_unit(u);
                        // `valid_filter` is asked of each unit that is active and on the map, in order; a no is a unit the
                        // filter drops, which the crate spells as its one exemption, then as an off-map unit.
                        if active && on_map && filtered {
                            let ans = a[12 + calls.min(7)];
                            calls += 1;
                            if ans == 0 {
                                if exempt.is_none() {
                                    exempt = Some(idx);
                                } else {
                                    s.units[idx].on_map = false;
                                }
                            }
                        }
                    }
                }
                let r = if a[4] < 0 { 1 << 28 } else { a[4] };
                let hit = s.find_unit_with_radius(r, Pos::new(a[0], a[1]), exempt);
                // The crate answers whether some unit qualifies; the original's index and nearest distance are not
                // modelled: held to what the bool implies, and the two words laid as the original wrote them.
                let wc = unhex(after.trim().split(';').nth(1).expect("c0 after"));
                let d = rd(&wc, 0x1fc);
                let who = rd(&wc, 0x200);
                if hit {
                    assert!(
                        d >= 0 && (a[4] < 0 || d <= a[4]),
                        "find_unit_with_radius — hit, 0x1fc {d}, row `{line}`"
                    );
                    assert!(
                        (0..4).contains(&who),
                        "find_unit_with_radius — hit, who {who}, row `{line}`"
                    );
                } else {
                    assert_eq!(
                        d, 99_999_999,
                        "find_unit_with_radius — miss, 0x1fc, row `{line}`"
                    );
                    assert_eq!(
                        who, a[3],
                        "find_unit_with_radius — miss, 0x200, row `{line}`"
                    );
                }
                wr(&mut c[0], 0x1fc, d);
                wr(&mut c[0], 0x200, who);
                if hit == (want_eax >= 0) {
                    want_eax
                } else {
                    i64::MIN
                }
            }
        };
        assert_eq!(got, want_eax, "find_unit_with_radius — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_unit_with_radius — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_unit_with_radius — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "find_unit_with_radius — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("ObjectsData::find_unit_with_radius@00659890: {n} rows agree");
}
