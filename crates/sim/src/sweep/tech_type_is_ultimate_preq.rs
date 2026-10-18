//! `TechType::is_ultimate_preq@0066c410` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

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

/// `TechType::is_ultimate_preq@0066c410`, run by `tools/emu/sweep_tech_type_is_ultimate_preq.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), `p` windows in order: p0..p43 the place(s), in flag order; the global cell 0x00c061e8 points at 2112 random bytes; the word at `0xe85ddc` points at 2516 random bytes less `0` (placed); the word at `p0+256` points at 464 random bytes less `0` (placed); the word at `p0+260` points at 464 random bytes less `0` (placed); the word at `p0+1680` points at 464 random bytes less `0` (placed); the word at `p0+1684` points at 464 random bytes less `0` (placed); the word at `p0+2048` points at 464 random bytes less `0` (placed); the word at `p0+2368` points at 464 random bytes less `0` (placed); the word at `p0+2372` points at 464 random bytes less `0` (placed); the word at `p0+2176` points at 464 random bytes less `0` (placed); the word at `p0+2180` points at 464 random bytes less `0` (placed); the word at `p0+2184` points at 464 random bytes less `0` (placed); the word at `p0+2188` points at 464 random bytes less `0` (placed); the word at `p0+2192` points at 464 random bytes less `0` (placed); the word at `p0+2196` points at 464 random bytes less `0` (placed); the word at `p0+2200` points at 464 random bytes less `0` (placed); the word at `p0+2204` points at 464 random bytes less `0` (placed); the word at `p0+2208` points at 464 random bytes less `0` (placed); the word at `p0+2212` points at 464 random bytes less `0` (placed); the word at `p0+2216` points at 464 random bytes less `0` (placed); the word at `p0+2220` points at 464 random bytes less `0` (placed); the word at `p0+2224` points at 464 random bytes less `0` (placed); the word at `p0+2228` points at 464 random bytes less `0` (placed); the word at `p0+2232` points at 464 random bytes less `0` (placed); the word at `p0+2236` points at 464 random bytes less `0` (placed); the word at `p0+2240` points at 464 random bytes less `0` (placed); the word at `p0+2244` points at 464 random bytes less `0` (placed); the word at `p0+2248` points at 464 random bytes less `0` (placed); the word at `p0+2252` points at 464 random bytes less `0` (placed); the word at `p0+2256` points at 464 random bytes less `0` (placed); the word at `p0+2260` points at 464 random bytes less `0` (placed); the word at `p0+2264` points at 464 random bytes less `0` (placed); the word at `p0+2268` points at 464 random bytes less `0` (placed); the word at `p0+2272` points at 464 random bytes less `0` (placed); the word at `p0+2276` points at 464 random bytes less `0` (placed); the word at `p0+2280` points at 464 random bytes less `0` (placed); the word at `p0+2284` points at 464 random bytes less `0` (placed); the word at `p0+2288` points at 464 random bytes less `0` (placed); the word at `p0+2292` points at 464 random bytes less `0` (placed); the word at `p0+2296` points at 464 random bytes less `0` (placed); the word at `p0+2300` points at 464 random bytes less `0` (placed); the word at `p0+2304` points at 464 random bytes less `0` (placed); the word at `p0+2308` points at 464 random bytes less `0` (placed); the word at `p0+2312` points at 464 random bytes less `0` (placed); the word at `0xc0aabc` points at 2516 random bytes less `0` (placed); the word at `p43+256` laid as the address of `p1` every row; the word at `p43+260` laid as the address of `p2` every row; the word at `p43+1680` laid as the address of `p3` every row; the word at `p43+1684` laid as the address of `p4` every row; the word at `p43+2048` laid as the address of `p5` every row; the word at `p43+2368` laid as the address of `p6` every row; the word at `p43+2372` laid as the address of `p7` every row; the word at `p43+2176` laid as the address of `p8` every row; the word at `p43+2180` laid as the address of `p9` every row; the word at `p43+2184` laid as the address of `p10` every row; the word at `p43+2188` laid as the address of `p11` every row; the word at `p43+2192` laid as the address of `p12` every row; the word at `p43+2196` laid as the address of `p13` every row; the word at `p43+2200` laid as the address of `p14` every row; the word at `p43+2204` laid as the address of `p15` every row; the word at `p43+2208` laid as the address of `p16` every row; the word at `p43+2212` laid as the address of `p17` every row; the word at `p43+2216` laid as the address of `p18` every row; the word at `p43+2220` laid as the address of `p19` every row; the word at `p43+2224` laid as the address of `p20` every row; the word at `p43+2228` laid as the address of `p21` every row; the word at `p43+2232` laid as the address of `p22` every row; the word at `p43+2236` laid as the address of `p23` every row; the word at `p43+2240` laid as the address of `p24` every row; the word at `p43+2244` laid as the address of `p25` every row; the word at `p43+2248` laid as the address of `p26` every row; the word at `p43+2252` laid as the address of `p27` every row; the word at `p43+2256` laid as the address of `p28` every row; the word at `p43+2260` laid as the address of `p29` every row; the word at `p43+2264` laid as the address of `p30` every row; the word at `p43+2268` laid as the address of `p31` every row; the word at `p43+2272` laid as the address of `p32` every row; the word at `p43+2276` laid as the address of `p33` every row; the word at `p43+2280` laid as the address of `p34` every row; the word at `p43+2284` laid as the address of `p35` every row; the word at `p43+2288` laid as the address of `p36` every row; the word at `p43+2292` laid as the address of `p37` every row; the word at `p43+2296` laid as the address of `p38` every row; the word at `p43+2300` laid as the address of `p39` every row; the word at `p43+2304` laid as the address of `p40` every row; the word at `p43+2308` laid as the address of `p41` every row; the word at `p43+2312` laid as the address of `p42` every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tech_type_is_ultimate_preq.py") else {
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
                // The record table: type position -> its `p` window. The tree is laid from the windows the way the data fixes it
                // (a position's kind is the position's: units 0x32.., buildings 0x19e.., the seven ages, the 28 library epochs
                // four lines of seven, the plain techs), then `TechTree::is_ultimate_preq` is asked.
                use crate::tech::{Kind, Line, Preq, Rules, Setup, TechTree, TypeDef, UnitTraits};
                const RECS: &[(usize, usize)] = &[
                    (0x40, 1),
                    (0x41, 2),
                    (0x1a4, 3),
                    (0x1a5, 4),
                    (0x200, 5),
                    (0x250, 6),
                    (0x251, 7),
                    (0x220, 8),
                    (0x221, 9),
                    (0x222, 10),
                    (0x223, 11),
                    (0x224, 12),
                    (0x225, 13),
                    (0x226, 14),
                    (0x227, 15),
                    (0x228, 16),
                    (0x229, 17),
                    (0x22a, 18),
                    (0x22b, 19),
                    (0x22c, 20),
                    (0x22d, 21),
                    (0x22e, 22),
                    (0x22f, 23),
                    (0x230, 24),
                    (0x231, 25),
                    (0x232, 26),
                    (0x233, 27),
                    (0x234, 28),
                    (0x235, 29),
                    (0x236, 30),
                    (0x237, 31),
                    (0x238, 32),
                    (0x239, 33),
                    (0x23a, 34),
                    (0x23b, 35),
                    (0x23c, 36),
                    (0x23d, 37),
                    (0x23e, 38),
                    (0x23f, 39),
                    (0x240, 40),
                    (0x241, 41),
                    (0x242, 42),
                ];
                let word = |w: &[u8], off: usize| rd(w, off);
                let g = &c[0];
                let setup = Setup {
                    starting_age: i32::from(g[0x34]),
                    starting_age2: i32::from(g[0x35]),
                    ending: i32::from(g[0x36]),
                    rules: if g[0x2a] == 8 {
                        Rules::BarbariansAtTheGates
                    } else {
                        Rules::Standard
                    },
                    ..Setup::STANDARD
                };
                let mut tree = TechTree::new();
                for pos in 0..0x275usize {
                    let kind = match pos {
                        0..=0x31 => Kind::Good,
                        0x32..=0x19d => Kind::Unit(UnitTraits::default()),
                        0x19e..=0x21f => Kind::Building {
                            auto: false,
                            wonder: false,
                        },
                        0x220..=0x226 => Kind::Age((pos - 0x220) as u8),
                        0x227..=0x242 => Kind::Epoch {
                            line: Line::ALL[3 - (pos - 0x227) / 7],
                            level: ((pos - 0x227) % 7) as u8,
                        },
                        0x243..=0x246 => Kind::Final,
                        0x247..=0x26e => Kind::Plain,
                        _ => Kind::Gov {
                            tier: ((pos - 0x26f) / 2) as u8,
                            column: ((pos - 0x26f) % 2) as u8,
                        },
                    };
                    tree.add(TypeDef::new("", kind));
                }
                tree.roles.university = Some(0x1a4);
                let to_preq = |v: i32| match v {
                    -2 => Preq::Disabled,
                    v if v < 0 => Preq::None,
                    v => Preq::Of(v as usize),
                };
                for &(pos, w) in RECS {
                    let r = &p[w];
                    assert_eq!(
                        word(r, 4),
                        pos as i32,
                        "record {pos:#x} names its own position"
                    );
                    let def = &mut tree.types[pos];
                    def.preq = [
                        to_preq(word(r, 0x30)),
                        to_preq(word(r, 0x34)),
                        to_preq(word(r, 0x38)),
                    ];
                    def.age = word(r, 0x1c8);
                    let wh = word(r, 0x40);
                    def.where_ = if wh < 0 { None } else { Some(wh as usize) };
                }
                let t = rd(m, 4);
                i64::from(tree.is_ultimate_preq(
                    &setup,
                    if t < 0 { usize::MAX } else { t as usize },
                    a[0] as usize,
                    0,
                ))
            }
        };
        assert_eq!(got, want_eax, "is_ultimate_preq — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_ultimate_preq — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_ultimate_preq — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "is_ultimate_preq — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("TechType::is_ultimate_preq@0066c410: {n} rows agree");
}
