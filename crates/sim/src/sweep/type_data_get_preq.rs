//! `TypeData::get_preq@00668700` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

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

use crate::tech::{Kind, Line, Preq, Rules, Setup, TechTree, TypeDef, UnitTraits};

/// The type list the original's `types.list` and `techtypes` are, as the
/// crate's tree: the index ranges `is_unit_type` / `is_age_type` /
/// `is_epoch_type` / `is_gov_type` test, the library blocks by `cat`
/// (`0x23c` Military, `0x235` Civic, `0x22e` Commerce, `0x227` Science), and
/// each record's `+0x1c8` age from the window the sweep laid at that index.
/// Records stand at `0x21f..=0x242`, the window `p[1 + index - 0x21f]`.
fn tree_from(c: &[Vec<u8>], p: &[Vec<u8>], this: &[u8]) -> (TechTree, usize) {
    let mut tree = TechTree::new();
    for idx in 0..0x275usize {
        let kind = match idx {
            0..=0x31 => Kind::Good,
            0x32..=0x19d => Kind::Unit(UnitTraits::default()),
            0x19e..=0x21e => Kind::Building {
                auto: false,
                wonder: false,
            },
            0x21f => Kind::Plain,
            0x220..=0x226 => Kind::Age((idx - 0x220) as u8),
            0x227..=0x242 => {
                let j = (idx - 0x227) / 7;
                Kind::Epoch {
                    line: Line::of(3 - j as i32).expect("a line"),
                    level: ((idx - 0x227) % 7) as u8,
                }
            }
            0x243..=0x246 => Kind::Final,
            0x247..=0x26e => Kind::Plain,
            _ => Kind::Gov {
                tier: ((idx - 0x26f) / 2) as u8,
                column: ((idx - 0x26f) % 2) as u8,
            },
        };
        let mut def = TypeDef::new("t", kind);
        if (0x21f..=0x242).contains(&idx) {
            def.age = rd(&p[1 + idx - 0x21f], 0x1c8);
        }
        tree.add(def);
    }
    tree.roles.university = Some(0x1a4);
    tree.iroquois_govs_early = rd(&c[1], 0x82c);
    let t = rd(this, 4) as usize;
    let preq = |v: i32| {
        if v < 0 {
            Preq::None
        } else {
            Preq::Of(v as usize)
        }
    };
    tree.types[t].preq = [
        preq(rd(this, 0x30)),
        preq(rd(this, 0x34)),
        preq(rd(this, 0x38)),
    ];
    let w = rd(this, 0x40);
    tree.types[t].where_ = if (0..0x275).contains(&w) {
        Some(w as usize)
    } else {
        None
    };
    (tree, t)
}

/// What the port answers, as the original's integer: `none` is `-1`.
fn port(a: &[i32], m: &[u8], c: &[Vec<u8>], p: &[Vec<u8>]) -> i64 {
    let (tree, t) = tree_from(c, p, m);
    let setup = Setup {
        starting_age: i32::from(c[0][0x34]),
        starting_age2: i32::from(c[0][0x35]),
        ending: i32::from(c[0][0x36]),
        rules: if c[0][0x2a] == 8 {
            Rules::BarbariansAtTheGates
        } else {
            Rules::Standard
        },
        ..Setup::STANDARD
    };
    // `param_2`: a leader index, or `-1`; `has_tribe_bonus(.., 0x12)` is the stub's answer.
    let who = (a[1] >= 0).then(|| {
        let mut who = crate::tech::PlayerTech::new(&tree);
        who.power = Some(if a[2] != 0 { 18 } else { 0 });
        who.has_city = true;
        who
    });
    match tree.get_preq(&setup, who.as_ref(), t, a[0] as u32 as usize) {
        Preq::None => -1,
        Preq::Of(x) => x as i64,
        Preq::Disabled => unreachable!("the sweep lays no disabled slot"),
    }
}

/// `TypeData::get_preq@00668700`, run by `tools/emu/sweep_type_data_get_preq.py` under unicorn
/// (tools/emu): 72 random bytes of object, 2 argument(s), `p` windows in order: p0..p37 the place(s), in flag order; the global cell 0x00c061e8 points at 64 random bytes; the global cell 0x00c061e4 points at 2112 random bytes; callee 0x006e1370 (1 args) stubbed; the word at `0xe85ddc` points at 144 random bytes less `0x87c` (placed); the word at `p0+0` points at 464 random bytes less `0` (placed); the word at `p0+4` points at 464 random bytes less `0` (placed); the word at `p0+8` points at 464 random bytes less `0` (placed); the word at `p0+12` points at 464 random bytes less `0` (placed); the word at `p0+16` points at 464 random bytes less `0` (placed); the word at `p0+20` points at 464 random bytes less `0` (placed); the word at `p0+24` points at 464 random bytes less `0` (placed); the word at `p0+28` points at 464 random bytes less `0` (placed); the word at `p0+32` points at 464 random bytes less `0` (placed); the word at `p0+36` points at 464 random bytes less `0` (placed); the word at `p0+40` points at 464 random bytes less `0` (placed); the word at `p0+44` points at 464 random bytes less `0` (placed); the word at `p0+48` points at 464 random bytes less `0` (placed); the word at `p0+52` points at 464 random bytes less `0` (placed); the word at `p0+56` points at 464 random bytes less `0` (placed); the word at `p0+60` points at 464 random bytes less `0` (placed); the word at `p0+64` points at 464 random bytes less `0` (placed); the word at `p0+68` points at 464 random bytes less `0` (placed); the word at `p0+72` points at 464 random bytes less `0` (placed); the word at `p0+76` points at 464 random bytes less `0` (placed); the word at `p0+80` points at 464 random bytes less `0` (placed); the word at `p0+84` points at 464 random bytes less `0` (placed); the word at `p0+88` points at 464 random bytes less `0` (placed); the word at `p0+92` points at 464 random bytes less `0` (placed); the word at `p0+96` points at 464 random bytes less `0` (placed); the word at `p0+100` points at 464 random bytes less `0` (placed); the word at `p0+104` points at 464 random bytes less `0` (placed); the word at `p0+108` points at 464 random bytes less `0` (placed); the word at `p0+112` points at 464 random bytes less `0` (placed); the word at `p0+116` points at 464 random bytes less `0` (placed); the word at `p0+120` points at 464 random bytes less `0` (placed); the word at `p0+124` points at 464 random bytes less `0` (placed); the word at `p0+128` points at 464 random bytes less `0` (placed); the word at `p0+132` points at 464 random bytes less `0` (placed); the word at `p0+136` points at 464 random bytes less `0` (placed); the word at `p0+140` points at 464 random bytes less `0` (placed); the word at `0xc0aabc` points at 144 random bytes less `0x87c` (placed); the word at `p37+0` laid as the address of `p1` every row; the word at `p37+4` laid as the address of `p2` every row; the word at `p37+8` laid as the address of `p3` every row; the word at `p37+12` laid as the address of `p4` every row; the word at `p37+16` laid as the address of `p5` every row; the word at `p37+20` laid as the address of `p6` every row; the word at `p37+24` laid as the address of `p7` every row; the word at `p37+28` laid as the address of `p8` every row; the word at `p37+32` laid as the address of `p9` every row; the word at `p37+36` laid as the address of `p10` every row; the word at `p37+40` laid as the address of `p11` every row; the word at `p37+44` laid as the address of `p12` every row; the word at `p37+48` laid as the address of `p13` every row; the word at `p37+52` laid as the address of `p14` every row; the word at `p37+56` laid as the address of `p15` every row; the word at `p37+60` laid as the address of `p16` every row; the word at `p37+64` laid as the address of `p17` every row; the word at `p37+68` laid as the address of `p18` every row; the word at `p37+72` laid as the address of `p19` every row; the word at `p37+76` laid as the address of `p20` every row; the word at `p37+80` laid as the address of `p21` every row; the word at `p37+84` laid as the address of `p22` every row; the word at `p37+88` laid as the address of `p23` every row; the word at `p37+92` laid as the address of `p24` every row; the word at `p37+96` laid as the address of `p25` every row; the word at `p37+100` laid as the address of `p26` every row; the word at `p37+104` laid as the address of `p27` every row; the word at `p37+108` laid as the address of `p28` every row; the word at `p37+112` laid as the address of `p29` every row; the word at `p37+116` laid as the address of `p30` every row; the word at `p37+120` laid as the address of `p31` every row; the word at `p37+124` laid as the address of `p32` every row; the word at `p37+128` laid as the address of `p33` every row; the word at `p37+132` laid as the address of `p34` every row; the word at `p37+136` laid as the address of `p35` every row; the word at `p37+140` laid as the address of `p36` every row; stub 0 answers 0..2 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_get_preq.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            port(a, m, c, p)
        };
        assert_eq!(got, want_eax, "get_preq — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_preq — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_preq — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "get_preq — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("TypeData::get_preq@00668700: {n} rows agree");
}
