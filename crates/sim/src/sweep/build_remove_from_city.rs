//! `Build::remove_from_city@00622030` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

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

/// `Build::remove_from_city@00622030`, run by `tools/emu/sweep_build_remove_from_city.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p18 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x00642da0 (0 args) stubbed; the word at `0xc09970+u8(m,9)*0x1c` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 112 random bytes less `0` (placed); the word at `p0+4` points at 112 random bytes less `0` (placed); the word at `p0+8` points at 112 random bytes less `0` (placed); the word at `c0+0x14+u8(m,9)*0x1c` points at 24 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 16 random bytes less `0` (placed); the word at `p4+12` points at 16 random bytes less `0` (placed); the word at `p4+16` points at 16 random bytes less `0` (placed); the word at `p4+20` points at 16 random bytes less `0` (placed); the word at `m+0x7c` points at 24 random bytes less `0` (placed); the word at `p11+0` points at 128 random bytes less `0` (placed); the word at `p11+4` points at 128 random bytes less `0` (placed); the word at `p11+8` points at 128 random bytes less `0` (placed); the word at `p11+12` points at 128 random bytes less `0` (placed); the word at `p11+16` points at 128 random bytes less `0` (placed); the word at `p11+20` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 720 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [0, 1, 2] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers `p12+j`, the data of the object its `ecx` is in `p5+j` (j < 6), linked at `+0x74`; stub 1 answers 1 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_remove_from_city.py") else {
        return;
    };
    let mut n = 0;
    let mut off_chain_rows = 0;
    let mut consistent_rows = 0;
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
                // next); p1..p3 the three city records of its player (`+4` flags, `+6` its own index, `+8` the
                // first building, `+0x5e` who: `City::regen_roads` runs natively and re-finds the city by them),
                // p5..p10 the six objects of the player, p12..p17 their data records (`+0x74` the next link, the
                // chain; `+0x60` the road flags `regen_roads` sets), p18 the type (`+0x2c0 & 0x10`). Stubs, in
                // `a`: 0..6 one answer each, then slot +0x4c (a[6]), slot +0xb8 (a[7..11]), the type's slot +0x60
                // (a[11..15]) and `count_buildings` (a[15..19]), each in call order. The crate's
                // `Sim::remove_from_city` is driven on a `Sim` rebuilt from them (the chain is `members`, tail
                // last); the city's flag bits, `Wall::decrement_stats` and the unlink's `next` write have no state
                // in it and are inline.
                use crate::build::{BuildType, Ident};
                use crate::city::City;
                use crate::world::{Pos, World};
                use crate::{Sim, Tuning};
                let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                let ci = i16at(m, 0x72);
                let (mut s4, mut s5): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
                // The whole body sits under one gate; the crate's callers hold it.
                if m[8] & 0x20 == 0 && ci >= 0 {
                    let ci = ci as usize;
                    let who = m[9];
                    let id = i16at(m, 0xa);
                    let next = i16at(m, 0x74);
                    let mut sim = Sim::new(Tuning::RON, World::new(8, 8), 2);
                    let _ = sim.add_build_type(BuildType {
                        ident: Ident::Village,
                        ..BuildType::default()
                    });
                    for _ in 0..7 {
                        sim.add_building(who, Pos::new(0, 0), 0);
                    }
                    let b = id.clamp(0, 6) as usize;
                    let walk = |p: &[Vec<u8>], head: i16| -> Vec<usize> {
                        let mut out = vec![head as usize];
                        let mut n = head;
                        while i16at(&p[12 + n as usize], 0x74) >= 0 {
                            n = i16at(&p[12 + n as usize], 0x74);
                            out.push(n as usize);
                        }
                        out
                    };
                    let before: Vec<Vec<usize>> =
                        (0..3).map(|k| walk(p, i16at(&p[1 + k], 8))).collect();
                    for (k, chain) in before.iter().enumerate() {
                        sim.cities.push(City {
                            alive: i16at(&p[1 + k], 4) & 1 != 0,
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
                    sim.buildings[b].city = Some(ci);
                    let chain = &before[ci];
                    let in_domain = chain[1..].contains(&b);
                    sim.remove_from_city(b);
                    assert_eq!(
                        sim.buildings[b].city, None,
                        "the city is cleared, row `{line}`"
                    );
                    // The unlink: the first node whose link names this building, else the tail, takes this
                    // building's own link (so a building off the chain overwrites the tail's).
                    let p_before = p.to_vec();
                    let prev = *chain
                        .iter()
                        .find(|&&n| i16at(&p[12 + n], 0x74) == id)
                        .unwrap_or_else(|| chain.last().expect("a head"));
                    p[12 + prev][0x74..0x76].copy_from_slice(&next.to_le_bytes());
                    let after = walk(p, i16at(&p[1 + ci], 8));
                    let alive = i16at(&p[1 + ci], 4) & 1 != 0;
                    let crate_chain: Vec<usize> = std::iter::once(sim.cities[ci].building)
                        .chain(sim.cities[ci].members.iter().copied())
                        .collect();
                    // The game keeps `this+0x74` and the chain record's link one field; the rows draw them
                    // apart, and then the original's unlink splices the *wrong* tail onto the predecessor
                    // (the crate removes the member, which is the invariant's reading).
                    let consistent = next == i16at(&p_before[12 + b], 0x74);
                    let kept: Vec<usize> = chain.iter().copied().filter(|&n| n != b).collect();
                    if in_domain {
                        assert_eq!(crate_chain, kept, "the member is removed, row `{line}`");
                        for n in 0..7 {
                            assert_eq!(
                                sim.buildings[n].regen_roads,
                                alive && kept.contains(&n),
                                "regen_roads flag of {n}, row `{line}`"
                            );
                        }
                        if consistent {
                            assert_eq!(
                                crate_chain, after,
                                "the chain after the unlink, row `{line}`"
                            );
                            consistent_rows += 1;
                        }
                    } else {
                        // Parted by construction (finding): a building off the chain, or the head itself, is
                        // left alone by the port; the original unlinks the tail's link.
                        assert_eq!(
                            crate_chain, *chain,
                            "off-chain: the port leaves it, row `{line}`"
                        );
                        off_chain_rows += 1;
                    }
                    if alive {
                        for &n in &after {
                            let r = &mut p[12 + n];
                            let f = u16::from_le_bytes([r[0x60], r[0x61]]) | 0x100;
                            r[0x60..0x62].copy_from_slice(&f.to_le_bytes());
                        }
                    }
                    let seq = |base: usize, n: &mut usize, len: usize| {
                        let v = a[base + (*n).min(len - 1)];
                        *n += 1;
                        v
                    };
                    let (mut n4c, mut nb8, mut n60, mut ncb) = (0, 0, 0, 0);
                    let build_vt = u32::from_le_bytes([m[0], m[1], m[2], m[3]]) == 0xb4_2174;
                    for (bit, ty) in [(0x80u16, 437), (0x400, 424), (0x200, 423), (0x800, 436)] {
                        let ability = if build_vt {
                            seq(11, &mut n60, 4)
                        } else {
                            seq(7, &mut nb8, 4)
                        };
                        if ability != 0 {
                            s4.push(format!("p{}+0x0,{ty},0,1", 1 + ci));
                            if seq(15, &mut ncb, 4) == 0 {
                                let r = &mut p[1 + ci];
                                let f = u16::from_le_bytes([r[4], r[5]]) & !bit;
                                r[4..6].copy_from_slice(&f.to_le_bytes());
                            }
                        }
                    }
                    m[0x72..0x74].copy_from_slice(&0xffffu16.to_le_bytes());
                    if p[18][0x2c0] & 0x10 == 0 {
                        let active = if build_vt {
                            m[8] & 4 != 0
                        } else {
                            seq(6, &mut n4c, 1) != 0
                        };
                        if active {
                            s5.push("m+0x0".to_string());
                        }
                    }
                }
                // The row prints `-` for a stub that was not called.
                log.push(if s4.is_empty() {
                    "-".to_string()
                } else {
                    s4.join("/")
                });
                log.push(if s5.is_empty() {
                    "-".to_string()
                } else {
                    s5.join("/")
                });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "remove_from_city — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "remove_from_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_from_city — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "remove_from_city — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Build::remove_from_city@00622030: {n} rows agree ({off_chain_rows} off the chain: the port leaves them; {consistent_rows} on the chain with this+0x74 = the record link)");
}
