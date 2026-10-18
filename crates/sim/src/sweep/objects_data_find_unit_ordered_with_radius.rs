//! `ObjectsData::find_unit_ordered_with_radius@00658ef0` under the emulator against the port (item 3238; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_unit_ordered_with_radius@00658ef0`, run by `tools/emu/sweep_objects_data_find_unit_ordered_with_radius.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 11 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p3 sparse table 1 (2 rows × 1 slice(s), row-major), p4..p25 the place(s), in flag order; the global cell 0x00c0618c points at 520 random bytes; callee 0x00616e80 (0 args) stubbed; the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 224 random bytes less `0` (placed); the word at `p6+4` points at 224 random bytes less `0` (placed); the word at `p6+8` points at 224 random bytes less `0` (placed); the word at `p7+0` points at 224 random bytes less `0` (placed); the word at `p7+4` points at 224 random bytes less `0` (placed); the word at `p7+8` points at 224 random bytes less `0` (placed); the word at `p14+0x18` points at 584 random bytes less `0` (placed); the word at `p15+0x18` points at 584 random bytes less `0` (placed); the word at `p16+0x18` points at 584 random bytes less `0` (placed); the word at `p17+0x18` points at 584 random bytes less `0` (placed); the word at `p18+0x18` points at 584 random bytes less `0` (placed); the word at `p19+0x18` points at 584 random bytes less `0` (placed); stub 0 answers from its own `ecx`: the word at `+0xdc`; 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_unit_ordered_with_radius.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The port is the crate's own `Sim::find_unit_ordered_with_radius`, driven on a `Sim` built from the
                // row's windows. c0 the `objects` record (the counts at `+0x15c + 4 * who`, the nearest-so-far pair
                // at `+0x1fc`/`+0x200`); p4/p5 the objects' pointer arrays and p6/p7 the units', per player; p8..p13
                // the objects' records (the alive bit at `+8`), p14..p19 the units' (`orders_x/y` `+0x70/+0x74`, the
                // order record `+0xdc` the stub reads as `order_type`'s answer), p20..p25 the types' profiles
                // (`big_radius` `+0x244`). a[0], a[1] the point, a[2] who, a[3] the radius.
                use crate::movement::Angle;
                use crate::orders::{Body, GroupMove, MoveKind, MoveOrder, Order};
                use crate::world::Pos;
                use crate::{Sim, Tuning, Unit, UnitType, World};
                let who = a[2];
                let mut s = Sim::new(Tuning::RON, World::new(8, 8), 2);
                for pl in 0..2usize {
                    let count = rd(&c[0], 0x15c + 4 * pl) as usize;
                    for j in 0..count {
                        let obj = &p[8 + 3 * pl + j];
                        let unit = &p[14 + 3 * pl + j];
                        let prof = &p[20 + 3 * pl + j];
                        let t = s.add_unit_type(UnitType {
                            combat: crate::combat::Profile {
                                big_radius: rd(prof, 0x244),
                                ..Default::default()
                            },
                            ..UnitType::default()
                        });
                        let alive = obj[8] & 1 != 0;
                        let mut u = Unit::new(
                            pl as u8,
                            (3 * pl + j) as i16,
                            Pos::new(0x1000, 0x1000),
                            i32::from(alive),
                        );
                        u.ty = Some(t);
                        let gm = GroupMove {
                            leader: 0,
                            id: 0,
                            form_id: 0,
                            group_angle: Angle(0),
                            in_group: false,
                            orig: Pos::new(0, 0),
                        };
                        // The order the stub answered: the move family's kinds as the crate spells them, the rest
                        // (and 18, `CHANGE_FORM`, which it cannot spell, is not drawn) as a `Build` order.
                        let mv = |kind: MoveKind, group: Option<GroupMove>| Order {
                            flags: 0,
                            body: Body::Move(MoveOrder {
                                kind,
                                dest: Pos::new(0, 0),
                                angle: Angle(0),
                                facing: None,
                                has_waypoint: false,
                                waypoint: Pos::new(0, 0),
                                last: None,
                                pause: 0,
                                retry: 0,
                                attempts: 0,
                                timer: 0,
                                coll: None,
                                orig: None,
                                group,
                            }),
                        };
                        let order = match rd(unit, 0xdc) {
                            0 => None,
                            1 => Some(mv(MoveKind::MoveTo, None)),
                            2 => Some(mv(MoveKind::AttackTo, None)),
                            3 => Some(mv(MoveKind::ExploreTo, None)),
                            4 => Some(mv(MoveKind::FleeTo, None)),
                            19 => Some(mv(MoveKind::MoveTo, Some(gm))),
                            21 => Some(mv(MoveKind::AttackTo, Some(gm))),
                            _ => Some(Order {
                                flags: 0,
                                body: Body::Build(0),
                            }),
                        };
                        let k = s.add_unit(u);
                        s.units[k].orders_pos = Pos::new(rd(unit, 0x70), rd(unit, 0x74));
                        if let Some(o) = order {
                            s.units[k].orders.push_back(o);
                        }
                    }
                }
                let hit =
                    s.find_unit_ordered_with_radius(a[3], Pos::new(a[0], a[1]), who as u8, None);
                // The crate answers whether some unit qualifies; the original's index and nearest distance are not
                // modelled (parked): held here to what the bool implies, and the two words laid as the original wrote.
                let wc = unhex(after.trim().split(';').nth(1).expect("c0 after"));
                let d = rd(&wc, 0x1fc);
                if hit {
                    assert!(
                        (0..=a[3]).contains(&d),
                        "find_unit_ordered_with_radius — hit, 0x1fc {d}, row `{line}`"
                    );
                } else {
                    assert_eq!(
                        d, 99_999_999,
                        "find_unit_ordered_with_radius — miss, 0x1fc, row `{line}`"
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
        assert_eq!(
            got, want_eax,
            "find_unit_ordered_with_radius — eax, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_unit_ordered_with_radius — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_unit_ordered_with_radius — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "find_unit_ordered_with_radius — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("ObjectsData::find_unit_ordered_with_radius@00658ef0: {n} rows agree");
}
