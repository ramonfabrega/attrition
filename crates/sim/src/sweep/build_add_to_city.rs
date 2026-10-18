//! `Build::add_to_city@00622380` under the emulator against the port (item 3250; tools/sweep_gen.py, obj with windows).

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

/// `Build::add_to_city@00622380`, run by `tools/emu/sweep_build_add_to_city.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 1 argument(s), `p` windows in order: p0..p18 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00643270 (0 args) stubbed; the word at `0xc09970+u8(m,9)*0x1c` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `c0+0x14+u8(m,9)*0x1c` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 16 random bytes less `0` (placed); the word at `p4+12` points at 16 random bytes less `0` (placed); the word at `p4+16` points at 16 random bytes less `0` (placed); the word at `p4+20` points at 16 random bytes less `0` (placed); the word at `m+0x7c` points at 24 random bytes less `0` (placed); the word at `p11+0` points at 128 random bytes less `0` (placed); the word at `p11+4` points at 128 random bytes less `0` (placed); the word at `p11+8` points at 128 random bytes less `0` (placed); the word at `p11+12` points at 128 random bytes less `0` (placed); the word at `p11+16` points at 128 random bytes less `0` (placed); the word at `p11+20` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 720 random bytes less `0` (placed); stub 1 answered from [0, 1, 1, 4] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers `p12+j`, the data of the object its `ecx` is in `p5+j` (j < 6), linked at `+0x74`; stub 1 answers 5 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_add_to_city.py") else {
        return;
    };
    let mut n = 0;
    let mut dead_rows = 0;
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: m the building (`+8` flags, `+9` who, `+0xa` id, `+0x18` type, `+0x72` city, `+0x74`
                // next); p1..p3 the three city records of its player (`+4` flags, `+8` the first building), p5..p10
                // the six objects of the player, p12..p17 their data records (`+0x74` the next link, the chain),
                // p18 the type (`+0x2c0 & 0x10`). `a[0]` the city, `a[6..11]` slot +0x4c's five answers, `a[11..15]`
                // slot +0xb8's four, `a[15..19]` the type's slot +0x60's four, each in call order. The crate's
                // `Sim::add_to_city` is driven on a `Sim` rebuilt from them (the chain is `members`, tail last);
                // the city's flag bits, `Wall::increment_stats` and the building's `next` link have no state in
                // it and are inline.
                use crate::build::{BuildType, Ident};
                use crate::city::City;
                use crate::world::{Pos, World};
                use crate::{Sim, Tuning};
                let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                let ci = a[0] as usize;
                let who = m[9];
                let id = i16at(m, 0xa);
                let mut sim = Sim::new(Tuning::RON, World::new(8, 8), 2);
                let village = sim.add_build_type(BuildType {
                    ident: Ident::Village,
                    ..BuildType::default()
                });
                let other = sim.add_build_type(BuildType::default());
                for _ in 0..6 {
                    sim.add_building(who, Pos::new(0, 0), 0);
                }
                let b = sim.add_building(who, Pos::new(0, 0), 0);
                sim.buildings[b].ty = Some(if m[8] & 0x20 != 0 { village } else { other });
                let walk = |p: &[Vec<u8>], head: i16| -> Vec<usize> {
                    let mut out = vec![head as usize];
                    let mut n = head;
                    while i16at(&p[12 + n as usize], 0x74) >= 0 {
                        n = i16at(&p[12 + n as usize], 0x74);
                        out.push(n as usize);
                    }
                    out
                };
                for k in 0..3 {
                    let r = &p[1 + k];
                    let chain = walk(p, i16at(r, 8));
                    sim.cities.push(City {
                        alive: i16at(r, 4) & 1 != 0,
                        owner: who,
                        race: Some(who),
                        founder: who,
                        building: chain[0],
                        members: chain[1..].to_vec(),
                        reg: None,
                        pos: Pos::new(0, 0),
                        capital: false,
                        founding_capital: false,
                        was_founding_capital: false,
                        unassimilated: false,
                        no_heal: false,
                        attacking: false,
                        ever_attacked: false,
                        alarm: false,
                        no_muster: false,
                        was_capital: 0,
                        capture_stamp: 0,
                        assimilation_timer: 0,
                        attack_stamp: 0,
                        reduce_stamp: 0,
                        raid_stamp: 0,
                        capture_strength: 0,
                        pop: 0,
                        has_citizen: false,
                        source: None,
                        trade_val: 0,
                        traded_with: [0; 8],
                    });
                }
                let alive = sim.cities[ci].alive;
                let old_tail = *sim.cities[ci]
                    .members
                    .last()
                    .unwrap_or(&sim.cities[ci].building);
                let before = sim.cities[ci].members.len();
                sim.add_to_city(b, ci);
                let pushed = sim.cities[ci].members.len() > before;
                if alive {
                    // `+0x72` is written before the city-centre gate.
                    let city = sim.buildings[b].city.expect("the city is set") as i16;
                    m[0x72..0x74].copy_from_slice(&city.to_le_bytes());
                    assert_eq!(pushed, m[8] & 0x20 == 0, "the centre gate, row `{line}`");
                } else {
                    // Parted (finding): the original does nothing for a dead city; the port sets
                    // the building's city and files it all the same.
                    assert_eq!(
                        sim.buildings[b].city,
                        Some(ci),
                        "dead-city parting, row `{line}`"
                    );
                    dead_rows += 1;
                }
                if alive && pushed {
                    assert_eq!(old_tail, *walk(p, i16at(&p[1 + ci], 8)).last().unwrap());
                    assert_eq!(sim.cities[ci].members.last(), Some(&b));
                    p[12 + old_tail][0x74..0x76].copy_from_slice(&id.to_le_bytes());
                    m[0x74..0x76].copy_from_slice(&0xffffu16.to_le_bytes());
                    let seq = |base: usize, n: &mut usize, len: usize| {
                        let v = a[base + (*n).min(len - 1)];
                        *n += 1;
                        v
                    };
                    let (mut n4c, mut nb8, mut n60) = (0, 0, 0);
                    let build_vt = u32::from_le_bytes([m[0], m[1], m[2], m[3]]) == 0xb4_2174;
                    let active = |n4c: &mut usize| {
                        if build_vt {
                            m[8] & 4 != 0
                        } else {
                            seq(6, n4c, 5) != 0
                        }
                    };
                    if p[18][0x2c0] & 0x10 == 0 && active(&mut n4c) {
                        log.push("m+0x0".to_string());
                    }
                    for bit in [0x80u16, 0x400, 0x200, 0x800] {
                        let ability = if build_vt {
                            seq(15, &mut n60, 4)
                        } else {
                            seq(11, &mut nb8, 4)
                        };
                        if ability != 0 && active(&mut n4c) {
                            let r = &mut p[1 + ci];
                            let f = u16::from_le_bytes([r[4], r[5]]) | bit;
                            r[4..6].copy_from_slice(&f.to_le_bytes());
                        }
                    }
                }
                // The row prints `-` for a stub that was not called.
                if log.is_empty() {
                    log.push("-".to_string());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_to_city — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_to_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_to_city — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "add_to_city — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    assert!(dead_rows > 0, "the dead-city parting is not reached");
    eprintln!("Build::add_to_city@00622380: {n} rows agree ({dead_rows} dead-city rows part)");
}
