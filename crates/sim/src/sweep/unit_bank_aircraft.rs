//! `Unit::bank_aircraft@005e9520` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

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

/// `Unit::bank_aircraft@005e9520`, run by `tools/emu/sweep_unit_bank_aircraft.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 1; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c061f0 points at 16 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 16 random bytes at 0x00e90000; 16 random bytes at 0x00e90100; 16 random bytes at 0x00e90200; 32 random bytes at 0x00e90300; 720 random bytes at 0x00e91000; 8 random bytes at 0x00e92000; 128 random bytes at 0x00e92100; argument 1 names 4 random bytes; vtable slot +0x30 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; callee 0x008544a0 (4 args) stubbed; stub 0 answered from [0, 1, 1] (90%); stub 1 answers 15270656..15270656 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 writes a value in -300..300 through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `c5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_bank_aircraft.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(10);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Unit::bank_aircraft`: the crate's `Sim::bank_plane_as` (item 3272), run on a `Sim` the row's windows
            // lay out. Windows: m the unit; c0 the game (frame at +0x550), c1 the constants (+8 unit_turn_speed),
            // c3 the order list's node (+4 the record), c4 the record (+8 the order, +0xc a byte), c6 the air
            // order base (+0x18 returning), c7 the type (+0x2b4 flags, +0x2c4 turn speed), c8 the guy's pointer
            // cell, c9 the guy (+0x14 z, +0x44 bank, +0x48 the bank before); p0 the speed. The args: des, the
            // speed's address, param_3 (0), then the `is_animal` slot's answer, the order slot's, the ground
            // stub's, and the ground height it writes.
            use crate::movement::Angle;
            use crate::single::Single;
            use crate::world::{Pos, World};
            let sh = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
            let animal = a[3] != 0;
            let heli = rd(&c[7], 0x2b4) & 0x20 != 0;
            let returning = rd(&c[6], 0x18) != 0;
            let owner = m[9];
            let k = 0x63637;
            let (px, py) = (rd(m, 0x10) ^ k, rd(m, 0x14) ^ k);
            let tile = Pos::new(px, py).tile();
            let mut w = World::new(40, 40);
            w.set_tile_z(tile, a[6]);
            let mut sim = crate::Sim::new(crate::Tuning::RON, w, 2);
            sim.tuning.unit_turn_speed = rd(&c[1], 8);
            sim.frame = i64::from(rd(&c[0], 0x550));
            let t = sim.add_unit_type(crate::UnitType::default());
            let mut unit = crate::Unit::new(1, 0, Pos::new(px, py), 1);
            unit.ty = Some(t);
            let u = sim.add_unit(unit);
            let st = sh(m, 0x98);
            {
                let un = &mut sim.units[u];
                un.owner = owner;
                un.spell_time = st;
                un.movement.heading = Angle(rd(m, 0x50));
                un.movement.turning.type_turn_speed = rd(&c[7], 0x2c4) as _;
                un.airframe.bank = Single::from_bits(rd(&c[9], 0x44) as u32);
                un.airframe.last_bank = Single::from_bits(rd(&c[9], 0x48) as u32);
                un.airframe.z = rd(&c[9], 0x14);
            }
            let mut speed = rd(&p[0], 0);
            let early = animal
                && Single::from_bits(rd(&c[9], 0x44) as u32).is_zero()
                && st >= 0
                && sim.frame & 7 == 0;
            sim.bank_plane_as(u, Angle(a[0]), &mut speed, returning, animal, heli);
            // The writes: the heading, the bank pair, the speed, and the order list's cache in `this`
            // (the node's record, the record's order and its byte), none of which an early return reaches.
            wr(m, 0x50, sim.units[u].movement.heading.0);
            wr(&mut c[9], 0x44, sim.units[u].airframe.bank.bits() as i32);
            wr(
                &mut c[9],
                0x48,
                sim.units[u].airframe.last_bank.bits() as i32,
            );
            wr(&mut p[0], 0, speed);
            if !early {
                let rec = rd(&c[3], 4);
                wr(m, 0xd4, rec);
                wr(m, 0xcc, rd(&c[4], 8));
                m[0xd0] = c[4][0xc];
            }
            // The ground stub is asked only by a flight that is not a returning plane's, from an owner under
            // eight, and with the tile the unit stands on.
            let asked = !early && owner < 8 && !(returning && !heli);
            let first: Vec<&str> = want_log[0].split(',').collect();
            if asked {
                log.push(if first.len() == 5 {
                    format!("{},{},{},{},0", first[0], first[1], tile.x, tile.y)
                } else {
                    "?".to_string()
                });
            } else {
                log.push("-".to_string());
            }
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "bank_aircraft — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "bank_aircraft — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "bank_aircraft — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[11 + i],
                "bank_aircraft — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::bank_aircraft@005e9520: {n} rows agree");
}
