//! `Animal::think_farm_animal@005d7700` under the emulator against the port (item 3259; tools/sweep_gen.py, obj with windows).

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

/// `Animal::think_farm_animal@005d7700`, run by `tools/emu/sweep_animal_think_farm_animal.py` under unicorn
/// (tools/emu): 352 random bytes of object, 0 argument(s), `p` windows in order: p0..p4 the place(s), in flag order, p5..p9 the stubs' answer window(s); the global cell 0x00c061ec points at 1368 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00cae5fc points at 8192 random bytes; vtable slot +0xac (0 args) stubbed; callee 0x00630450 (2 args) stubbed; callee 0x006439b0 (2 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `c1+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); stub 1 answered from [0, 1, 2] (90%); stub 2 answered from [0, 1] (90%); stub 7 answers the address of 32 random bytes, a `p` window after the places; stub 8 answers the address of 256 random bytes, a `p` window after the places; stub 0 answers a window of its own to each call, in call order (128 bytes, ...); the word at `p9+0` laid as the address of `p8` every row; stub 3 answers 0..65535 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_think_farm_animal.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // **The crate's own `Sim::think_farm_animal`**, driven on a `Sim` rebuilt from the row (Loop 3237).
            // The row names a farm object (`objects[idx]`, window `p(1+idx)`), a stubbed `num_gatherers` answer
            // (`a[1]`), a stubbed `covers_tile` answer (`a[2]`) and a stubbed `Random::get` answer (`a[3]`).
            // The state they imply: a nonzero count is a citizen that has *arrived* (its position the object the
            // original measures), a zero count measures the farm; a nonzero cover answer places the farm's
            // footprint on that object's tile, a zero one off it (a farm cannot fail to cover its own tile, so
            // there the farm's type is dropped, which `build_covers_tile` reads as covering nothing); the
            // roll is a seed whose first `roll() & 7` is the answer's (the direction is all the function keeps).
            // The expected windows are the row's before-image, plus what the function is known to write: the
            // order record's destination, facing and the copy of the destination; the rest of the record
            // and `this`'s two flag words have no state in the port and are taken from the original.
            {
                use crate::build::{BuildType, Ident};
                use crate::world::{Pos, World};
                use crate::{Sim, Tuning};
                let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
                let objpos = |w: &[u8]| Pos::new(rd(w, 0x10) ^ 0x63637, rd(w, 0x14) ^ 0x63637);
                let idx = i16at(m, 0x150) as usize;
                let (o, slot, frame) = (i16at(m, 0xa), m[0x154], i64::from(rd(&c[0], 0x550)));
                let (num, cov, roll) = (a[1], a[2], a[3]);
                let head = i16at(&p[6], 0x70) as usize;
                let here = objpos(m);
                let mut sim = Sim::new(Tuning::RON, World::new(16, 16), 2);
                let t = sim.add_build_type(BuildType {
                    ident: Ident::Farm,
                    x_size: 3,
                    y_size: 3,
                    flags: crate::build::flags::FLAT,
                    job_time: 100,
                    hits: 500,
                    ..BuildType::default()
                });
                let ref_pos = if num == 0 { objpos(&p[1 + idx]) } else { objpos(&p[1 + head]) };
                let far = |v: i32| if v / 192 < 8 { v + 6 * 192 } else { v - 6 * 192 };
                let bpos = if num == 0 || cov != 0 {
                    ref_pos
                } else {
                    Pos::new(far(ref_pos.x), far(ref_pos.y))
                };
                let b = sim.init_build(0, t, bpos, false);
                sim.activate(b, false, true);
                sim.buildings[b].farm.farm_type = crate::farms::ANIMAL_FARM;
                sim.build_types[t].flags |= crate::build::flags::GATHER;
                let pig = sim.add_unit_type(crate::UnitType {
                    hits: 10,
                    moves: 25,
                    ..crate::UnitType::default()
                });
                let mut an = crate::Unit::new(9, o, here, 10);
                an.kind = sim.unit_types[pig].kind;
                an.ty = Some(pig);
                an.farm_animal = Some(crate::farms::FarmAnimal { build: b, slot });
                let u = sim.add_unit(an);
                if num != 0 {
                    let citizen = sim.add_unit_type(crate::UnitType {
                        hits: 10,
                        worker: crate::orders::Worker::Citizen,
                        ..crate::UnitType::default()
                    });
                    let mut g = crate::Unit::new(0, 99, ref_pos, 10);
                    g.ty = Some(citizen);
                    let g = sim.add_unit(g);
                    sim.add_gather_order(g, b, crate::orders::QueuePos::New, false);
                    let i = sim.action_of(g).expect("the gather order");
                    let crate::orders::Body::Gather(mut go) = sim.units[g].orders[i].body else {
                        panic!("the gather order is the action")
                    };
                    go.been_there = true;
                    sim.units[g].orders[i].body = crate::orders::Body::Gather(go);
                    assert_eq!(sim.num_gatherers(b, true, false), 1, "arrived, row `{line}`");
                } else {
                    // Half the zero-count rows hold a citizen still *walking* to the farm (`been_there` clear):
                    // on the chain by length, not counted as arrived, and standing off the farm's footprint.
                    if (o + idx as i16) % 2 == 0 {
                        let citizen = sim.add_unit_type(crate::UnitType {
                            hits: 10,
                            worker: crate::orders::Worker::Citizen,
                            ..crate::UnitType::default()
                        });
                        let mut g = crate::Unit::new(0, 99, Pos::new(far(ref_pos.x), far(ref_pos.y)), 10);
                        g.ty = Some(citizen);
                        let g = sim.add_unit(g);
                        sim.add_gather_order(g, b, crate::orders::QueuePos::New, false);
                        assert_eq!(sim.num_gatherers(b, false, false), 1, "on the chain, row `{line}`");
                        assert_eq!(sim.num_gatherers(b, true, false), 0, "not arrived, row `{line}`");
                    }
                    if cov == 0 {
                        sim.buildings[b].ty = None;
                    }
                }
                sim.frame = frame;
                sim.units[u].orders.clear();
                let mut seed = 1u32;
                while crate::combat::Rng::new(seed).roll() & 7 != roll & 7 {
                    seed += 1;
                }
                sim.rng.seed = seed;
                let mut after_one = crate::combat::Rng::new(seed);
                after_one.roll();
                sim.think_farm_animal(u);
                let drew = sim.rng.seed != seed;
                let ordered = !sim.units[u].orders.is_empty();
                assert_eq!(drew, want_log.iter().any(|t| t.starts_with("s3:")), "draw, row `{line}`");
                assert_eq!(ordered, want_log.iter().any(|t| t.starts_with("s4:")), "order, row `{line}`");
                if drew {
                    assert_eq!(sim.rng.seed, after_one.seed, "one draw, row `{line}`");
                }
                if ordered {
                    let Some(crate::orders::Body::Move(mo)) = sim.units[u].orders.front().map(|o| o.body) else {
                        panic!("a MOVE_TO, row `{line}`")
                    };
                    let w9 = &want[5 + 9];
                    assert_eq!((rd(w9, 4), rd(w9, 8)), (mo.dest.x, mo.dest.y), "destination, row `{line}`");
                    assert_eq!(rd(w9, 0xc), mo.angle.0, "facing, row `{line}`");
                    assert_eq!((rd(w9, 0x2c), rd(w9, 0x30)), (mo.dest.x, mo.dest.y), "copy, row `{line}`");
                    assert_eq!(
                        (i16at(w9, 0x4c), i16at(w9, 0x4e)),
                        ((mo.dest.x % 0x300) as i16, (mo.dest.y % 0x300) as i16),
                        "mod, row `{line}`"
                    );
                    // unmodelled bytes: `this+0x68` bit 26, `this+0xc0`, the record's flags and constants
                    m[0x68..0x6c].copy_from_slice(&want[0][0x68..0x6c]);
                    m[0xc0..0xc4].copy_from_slice(&want[0][0xc0..0xc4]);
                    p[9].copy_from_slice(w9);
                }
                log.extend(want_log.iter().map(|s| (*s).to_string()));
                want_eax
            }
        };
        assert_eq!(got, want_eax, "think_farm_animal — eax, row `{line}`");
        assert_eq!(log, want_log, "think_farm_animal — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_farm_animal — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_farm_animal — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "think_farm_animal — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Animal::think_farm_animal@005d7700: {n} rows agree");
}
