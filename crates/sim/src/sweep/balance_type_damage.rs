//! `Balance::type_damage@0057fb50` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

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

use sim::Tuning;
use sim::attrition::Domain;
use sim::balance::{Kind, line as l, type_damage};

/// `ObjectTypeData::is(id, strict)` -> the crate's lineage bit (rondata `LINE_ROOTS`; MERCHANT and
/// CITIZEN are not `is` calls: the original compares the record's `TypeIndex`).
const LINES: [(i32, bool, u64); 41] = [
    (0xd8, false, l::ARMOREDCAR),
    (0xef, false, l::LIGHTTANK),
    (0x7b, false, l::MACHINEGUN),
    (0x83, false, l::FLAMETHROWER),
    (0x83, true, l::FLAMETHROWER_STRICT),
    (0x42, false, l::MILITIA),
    (0x15a, false, l::BOMBARDSHIP),
    (0x143, false, l::BARK),
    (0x152, false, l::SUB),
    (0x14e, false, l::FIRERAFT),
    (0x154, false, l::TRIREME),
    (0x130, false, l::BOMBER),
    (0x134, false, l::FIGHTERBOMBER),
    (0x136, false, l::HELICOPTER),
    (0x139, false, l::V2ROCKET),
    (0x3f, false, l::SUPPLYWAGON),
    (0x58, false, l::BALAMOBSLINGERS),
    (0xae, false, l::KUSHITEARCHERS),
    (0xae, true, l::KUSHITEARCHERS_STRICT),
    (0x5b, false, l::INTICLUBMEN),
    (0xbf, false, l::CAMELRANGE2),
    (0xc3, false, l::CHARIOT),
    (0xc7, false, l::NOMAD),
    (0xe0, false, l::RUSINYLANCER),
    (0xb1, true, l::LONGBOWMEN_STRICT),
    (0xb2, true, l::ELONGBOWMEN_STRICT),
    (0xb3, true, l::KINGSYEOMANRY_STRICT),
    (0xe8, false, l::ECOMPANION),
    (0x92, false, l::LEGIONS),
    (0xa0, true, l::SAMURAI_STRICT),
    (0x95, false, l::HALBERDIERS),
    (0x97, false, l::TERCIOS),
    (0x90, false, l::RECOILGUN),
    (0x72, false, l::HIGHLANDERS),
    (0x82, true, l::MG42_STRICT),
    (0x102, true, l::TIGERTANK_STRICT),
    (0x103, true, l::LEOPARDTANK_STRICT),
    (0x117, false, l::FLAMINGARROW),
    (0x114, false, l::BASILICABOMBARD),
    (0x112, false, l::MORTAR),
    (0x1bf, false, l::AIRBASE),
];

fn ushorts(w: &[u8], n: i32) -> Vec<i32> {
    (0..(n.max(0) as usize).min(w.len() / 2))
        .map(|i| i32::from(u16::from_le_bytes([w[2 * i], w[2 * i + 1]])))
        .collect()
}

/// A type record (its own bytes, its `is_list` buffer, its strict list's) as the crate's `Kind`.
fn kind_of(rec: &[u8], list: &[u8], strict: &[u8]) -> Kind {
    let idx = rd(rec, 4);
    let (is_list, strict_list) = (
        ushorts(list, rd(rec, 0x280)),
        ushorts(strict, rd(rec, 0x29c)),
    );
    let mut lines = 0u64;
    for (id, st, bit) in LINES {
        // `ObjectTypeData::is`: the index itself first, then the list (the strict one when asked).
        if idx == id || (if st { &strict_list } else { &is_list }).contains(&id) {
            lines |= bit;
        }
    }
    if matches!(idx, 0x3d | 0x3e | 0x190) {
        lines |= l::MERCHANT;
    }
    if (0x32..=0x35).contains(&idx) {
        lines |= l::CITIZEN;
    }
    // Bit 27 (missile) is the answer of slot +0x114, never the record's own word.
    let mut masks = rd(rec, 0x1e4) as u32 & !(1 << 27);
    if rd(rec, 0x2c8) != 0 {
        masks |= 1 << 27;
    }
    Kind {
        masks,
        age: rd(rec, 0x2bc),
        unit: rd(rec, 0x2b0) != 0,
        build: rd(rec, 0x2b4) != 0,
        wonder: rd(rec, 0x2b8) != 0,
        gaia: false,
        siege: rd(rec, 0x2c0) != 0,
        caravan: rd(rec, 0x2c4) != 0,
        domain: match rd(rec, 0x218) {
            0 => Domain::Land,
            1 => Domain::Sea,
            _ => Domain::Air,
        },
        lines,
        unit_index: None,
    }
}

/// `Balance::type_damage@0057fb50`, run by `tools/emu/sweep_balance_type_damage.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c061f0 points at 256 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x114 (0 args) stubbed; the word at `0xc061f8` points at 32 random bytes less `0` (placed); the word at `p0+0x10` points at 28 random bytes less `0` (placed); the word at `p1+20` points at 768 random bytes less `0` (placed); the word at `p1+24` points at 768 random bytes less `0` (placed); the word at `p2+0x28c` points at 16 random bytes less `0` (placed); the word at `p2+0x2a8` points at 16 random bytes less `0` (placed); the word at `p3+0x28c` points at 16 random bytes less `0` (placed); the word at `p3+0x2a8` points at 16 random bytes less `0` (placed); slot +0x60 laid as 0x0065f7d0 or its stub, a flag after the answers; stub 0 answers from its own `ecx`: the word at `+0x2b0`; stub 1 answers from its own `ecx`: the word at `+0x2b4`; stub 2 answers from its own `ecx`: the word at `+0x2b8`; stub 4 answers from its own `ecx`: the word at `+0x2bc`; stub 5 answers from its own `ecx`: the word at `+0x2c0`; stub 6 answers from its own `ecx`: the word at `+0x2c4`; stub 7 answers from its own `ecx`: the word at `+0x2c8`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_balance_type_damage.py") else {
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
            // The two type records are `p[2]` (the attacker, table slot 5) and `p[3]` (the target, slot
            // 6). The original reads their words and asks five stubbed slots, each answered from a
            // word of the record itself (`ecx+0x2b0` unit, `+0x2b4` build, `+0x2b8` wonder, `+0x2bc`
            // age, `+0x2c0` siege, `+0x2c4` caravan, `+0x2c8` missile); slot `+0x60` is the real
            // `ObjectTypeData::is@0065f7d0`, which reads `+4` (the index), the `is_list` (count `+0x280`,
            // `u16`s behind `+0x28c`) and the strict list (`+0x29c`, `+0x2a8`). So each record is a `Kind`.
            let mut t = Tuning::RON;
            t.one_age_down = rd(&c[0], 0x70);
            t.two_ages_down = rd(&c[0], 0x74);
            t.three_ages_down = rd(&c[0], 0x78);
            t.four_ages_down = rd(&c[0], 0x7c);
            t.five_ages_down = rd(&c[0], 0x80);
            let (ka, kb) = (kind_of(&p[2], &p[4], &p[5]), kind_of(&p[3], &p[6], &p[7]));
            i64::from(type_damage(&t, &ka, &kb))
        };
        assert_eq!(got, want_eax, "type_damage — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "type_damage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "type_damage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "type_damage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1800, "only {n} rows");
    eprintln!("Balance::type_damage@0057fb50: {n} rows agree");
}
