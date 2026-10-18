//! `ObjectData::has_nuke@00643d40` under the emulator against the port (item 3259; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::has_nuke@00643d40`, run by `tools/emu/sweep_object_data_has_nuke.py` under unicorn
/// What the original looked at: the building's owner and number, whether it was active, the queue entries it asked
/// about (each whether it named a missile), and the units it passed (or would have, as decoys).
struct Lay {
    owner: u8,
    idx: i32,
    active: bool,
    /// Each entry: whether it names a missile, and whether it is a tech (a type outside the unit range), a decoy
    /// of a missile type the crate must pass over.
    queue: Vec<(bool, bool)>,
    units: Vec<LayUnit>,
}

struct LayUnit {
    alive: bool,
    nuke: bool,
    /// The home of a `STRAFE` as its current order, if it has one.
    home: Option<i32>,
    inside: Option<i32>,
}

impl Lay {
    /// `Sim::has_nuke` (the crate's `ObjectData::has_nuke@00643d40`) on a `Sim` with the building at crate number
    /// `idx` (three buildings, all the owner's), a missile type and a plain one, the queue and the owner's units
    /// as laid, and a decoy missile of the other player homed at and inside the building.
    fn crate_says(&self) -> bool {
        use crate::combat::{Profile, mask};
        use crate::orders::{Body, Order, StrafeOrder};
        use crate::production::Item;
        use crate::{Pos, Sim, Tuning, Unit, UnitType};
        let mut sim = Sim::new(Tuning::RON, crate::world::World::new(8, 8), 2);
        let missile = sim.add_unit_type(UnitType {
            combat: Profile {
                obj_masks: mask::MISSILE,
                ..Profile::default()
            },
            ..UnitType::default()
        });
        let plain = sim.add_unit_type(UnitType::default());
        let owner = self.owner;
        for _ in 0..3 {
            sim.add_building(owner, Pos::new(0, 0), 4);
        }
        let b = usize::try_from(self.idx).expect("building number");
        sim.buildings[b].active = self.active;
        for &(hit, tech) in &self.queue {
            let mut it = Item::queued(
                if hit || tech { missile } else { plain },
                &[0; crate::economy::RESOURCES],
            );
            it.tech = tech.then_some(0);
            sim.buildings[b].queue.items.push(it);
        }
        let strafe = |home: Option<i32>| Order {
            flags: 0,
            body: Body::Strafe(StrafeOrder {
                target: None,
                mandatory: false,
                home: home.map(|h| usize::try_from(h).expect("home")),
                cruising_alt: 0,
                sharp_turn: 0,
                returning: true,
                at: None,
            }),
        };
        let mut lay = |own: u8, alive: bool, nuke: bool, home: Option<i32>, inside: Option<i32>| {
            let mut u = Unit::new(own, 0, Pos::new(0, 0), if alive { 100 } else { 0 });
            u.ty = Some(if nuke { missile } else { plain });
            u.inside = inside.map(|i| usize::try_from(i).expect("inside"));
            if home.is_some() {
                u.orders.push_back(strafe(home));
                if home != Some(self.idx) {
                    // Behind the current order, one homed at the building: only the front counts.
                    u.orders.push_back(strafe(Some(self.idx)));
                }
            }
            sim.add_unit(u);
        };
        for u in &self.units {
            lay(owner, u.alive, u.nuke, u.home, u.inside);
        }
        lay(1 - owner, true, true, Some(self.idx), Some(self.idx));
        sim.has_nuke(b)
    }
}

/// (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0..p40 the place(s), in flag order, p41..p42 the stubs' answer window(s); 8 random bytes at 0x00c0accc (each word in 0..2); vtable slot +0x10 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x114 (0 args) stubbed; callee 0x00651a80 (1 args) stubbed; the word at `0xc0ab84` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `0xc0aba0` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 32 random bytes less `0` (placed); the word at `p3+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 232 random bytes less `0` (placed); the word at `p6+4` points at 232 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p9+0` points at 232 random bytes less `0` (placed); the word at `p9+4` points at 232 random bytes less `0` (placed); the word at `p7+0x18` points at 512 random bytes less `0` (placed); the word at `p8+0x18` points at 512 random bytes less `0` (placed); the word at `p10+0x18` points at 512 random bytes less `0` (placed); the word at `p11+0x18` points at 512 random bytes less `0` (placed); the word at `p7+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p8+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p10+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p11+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p16+4` points at 16 random bytes less `0` (placed); the word at `p17+4` points at 16 random bytes less `0` (placed); the word at `p18+4` points at 16 random bytes less `0` (placed); the word at `p19+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 64 random bytes less `0` (placed); the word at `p21+8` points at 64 random bytes less `0` (placed); the word at `p22+8` points at 64 random bytes less `0` (placed); the word at `p23+8` points at 64 random bytes less `0` (placed); the word at `0xe85ddc` points at 20 random bytes less `-4` (placed); the word at `p28+0` points at 256 random bytes less `0` (placed); the word at `p28+4` points at 256 random bytes less `0` (placed); the word at `p28+8` points at 256 random bytes less `0` (placed); the word at `p28+12` points at 256 random bytes less `0` (placed); the word at `p28+16` points at 256 random bytes less `0` (placed); the word at `0xc0a274` points at 20 random bytes less `-4` (placed); the word at `p34+0` points at 512 random bytes less `0` (placed); the word at `p34+4` points at 512 random bytes less `0` (placed); the word at `p34+8` points at 512 random bytes less `0` (placed); the word at `p34+12` points at 512 random bytes less `0` (placed); the word at `p34+16` points at 512 random bytes less `0` (placed); the word at `0xc0a300` points at 60 random bytes less `0` (placed); stub 0 answered from [0, 1, 16, 16, 16, 2] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 1 answers the address of 152 random bytes, a `p` window after the places; stub 2 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p29+0` half the rows; the stubbed vtable laid at `p30+0` half the rows; the stubbed vtable laid at `p31+0` half the rows; the stubbed vtable laid at `p32+0` half the rows; the stubbed vtable laid at `p33+0` half the rows; the stubbed vtable laid at `p35+0` half the rows; the stubbed vtable laid at `p36+0` half the rows; the stubbed vtable laid at `p37+0` half the rows; the stubbed vtable laid at `p38+0` half the rows; the stubbed vtable laid at `p39+0` half the rows; the word at `p41+0x8c` laid as the address of `p40` every row; stub 0 answers 6 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 6 values in call order, printed after the answers; stub 5 answers -1..2 every row; stub 5 writes a value in [0, 1] through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_has_nuke.py") else {
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
                // `ObjectData::has_nuke`. `a`: 0..=1 the own arguments, 2..=7 the stubs' singles (unused), 8 what get_inside writes
                // through its argument, 9..=14 slot +0x10's run (0: the object active; then the unit's current order's kind),
                // 15..=18 slot +0xc's, 19..=22 slot +0x114's, 23..=28 get_inside's. c0 the unit counts (0xc0accc); p7/p8,
                // p10/p11 the owners' units, p12.. a unit's type, p16.. its order node, p20.. the node's record (+8 the order),
                // p29..p33 / p35..p39 types.list / unittypes of index -1..3, p40 the queue, p41 the building data (stub 1's
                // answer), p42 the order's home record (stub 2's). A type's vtable word is the image's (`0xb41fd4`) or the stubbed one.
                const UNIT_VT: i32 = 0xb41fd4;
                const NUKE: i32 = 0x8000000;
                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                let owner = usize::from(m[9]);
                let idx = sh(m, 10);
                let (mut k0, mut k3, mut k4, mut k5) = (0usize, 0usize, 0usize, 0usize);
                let mut res: i64 = 0;
                let mut lay = Lay {
                    owner: m[9],
                    idx,
                    active: false,
                    queue: Vec::new(),
                    units: Vec::new(),
                };
                'f: {
                    let active = a[9 + k0];
                    k0 += 1;
                    lay.active = active != 0;
                    if active == 0 {
                        // A missile queued in a building that is not active: the crate must not see it.
                        lay.queue.push((true, false));
                    }
                    if active != 0 {
                        let bd = &p[41];
                        let n = i32::from(bd[0x82]);
                        let mut i = 0;
                        while i < n {
                            let ty = if i < rd(bd, 0x88) {
                                sh(&p[40], 0x14 * usize::try_from(i).expect("entry") + 4)
                            } else {
                                -1
                            };
                            let slot = usize::try_from(ty + 1).expect("type -1..3");
                            let tw = &p[29 + slot];
                            let unit_type = if rd(tw, 0) == UNIT_VT {
                                let t = rd(tw, 4);
                                0x31 < t && t < 0x19e
                            } else {
                                k3 += 1;
                                a[15 + k3 - 1] != 0
                            };
                            let mut hit = false;
                            if unit_type {
                                let uw = &p[35 + slot];
                                let missile = if rd(uw, 0) == UNIT_VT {
                                    rd(uw, 0x1e4) & NUKE != 0
                                } else {
                                    k4 += 1;
                                    a[19 + k4 - 1] != 0
                                };
                                hit = missile;
                            }
                            lay.queue.push((hit, !unit_type));
                            if hit {
                                res = 1;
                                break 'f;
                            }
                            i += 1;
                        }
                    }
                    let cnt = rd(&c[0], 4 * owner);
                    for u in 0..cnt.max(0) as usize {
                        let k = owner * 2 + u;
                        let un = [7usize, 8, 10, 11][k];
                        let (alive, nuke) = (p[un][8] & 1 != 0, rd(&p[12 + k], 0x1e4) & NUKE != 0);
                        if !(alive && nuke) {
                            // A decoy the crate must pass over: homed at and inside the building all the same.
                            lay.units.push(LayUnit {
                                alive,
                                nuke,
                                home: Some(idx),
                                inside: Some(idx),
                            });
                            continue;
                        }
                        let mut lu = LayUnit {
                            alive,
                            nuke,
                            home: None,
                            inside: None,
                        };
                        if rd(&p[un], 0xdc) != 0 {
                            let (nd, rc) = (rd(&p[16 + k], 4), rd(&p[20 + k], 8));
                            let b = p[20 + k][0xc];
                            wr(&mut p[un], 0xd4, nd);
                            wr(&mut p[un], 0xcc, rc);
                            p[un][0xd0] = b;
                            let kind = a[9 + k0];
                            k0 += 1;
                            if kind == 0x10 {
                                lu.home = Some(sh(&p[42], 0x28));
                            }
                            if kind == 0x10 && sh(&p[42], 0x28) == idx {
                                lay.units.push(lu);
                                res = 1;
                                break 'f;
                            }
                        }
                        let inside = a[23 + k5];
                        k5 += 1;
                        if a[8] == owner as i32 && inside >= 0 {
                            lu.inside = Some(inside);
                        }
                        lay.units.push(lu);
                        if inside == idx && a[8] == owner as i32 {
                            res = 1;
                            break 'f;
                        }
                    }
                }
                // The crate's own `Sim::has_nuke` on a `Sim` laid from what the original looked at.
                assert_eq!(
                    i64::from(lay.crate_says()),
                    res,
                    "the crate's has_nuke, row `{line}`"
                );
                res
            }
        };
        assert_eq!(got, want_eax, "has_nuke — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_nuke — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_nuke — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "has_nuke — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("ObjectData::has_nuke@00643d40: {n} rows agree");
}
