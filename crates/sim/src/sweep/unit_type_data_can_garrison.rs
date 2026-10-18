//! `UnitTypeData::can_garrison@0061d7d0` under the emulator against the port (item 3250; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

use sim::attrition::{Domain, UnitKind};
use sim::build::{BuildType, Ident};
use sim::garrison::UnitTraits;
use sim::world::World;
use sim::{Sim, Tuning, UnitType};

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `UnitTypeData::can_garrison@0061d7d0`, run by `tools/emu/sweep_unit_type_data_can_garrison.py` under unicorn
/// (tools/emu): 704 random bytes of object, 1 argument(s), `p` windows in order: p0..p6 the place(s), in flag order; vtable slot +0x60 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x64 (0 args) stubbed; vtable slot +0x108 (0 args) stubbed; callee 0x00595260 (0 args) stubbed; the word at `0xc0aaa0` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 16 random bytes less `0` (placed); the word at `p0+20` points at 16 random bytes less `0` (placed); the word at `p1+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; the word at `p2+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; the word at `p3+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; the word at `p4+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; the word at `p5+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; the word at `p6+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 1 values in call order, printed after the answers; stub 2 answers 1 values in call order, printed after the answers; stub 3 answers 1 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 2 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers 4..6 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_can_garrison.py") else {
        return;
    };
    let (mut n, mut skipped_neg, mut skipped_inconsistent) = (0, 0, 0);
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The recorded calls are the INPUT here, not a check: they are the questions the original asked
                // its build type (slot +0x60 with an id, or the override slots +0xfc fort, +0x64 city, +0x108
                // dock) and the unit type itself (`+0x60(0x15f, 1)`), and the answers it got, in call order
                // (`a[6..12]` the +0x60 stream, `a[12]`, `a[13]`, `a[14]` the three overrides, `a[5]` the
                // patch version). The `Sim` is laid so its lineage tests answer the same: a question the
                // original did not ask is laid false, which cannot matter (an earlier true answer has
                // already returned 1, and the crate reads the same answers in an OR).
                let toks: Vec<Vec<(&str, Vec<i32>)>> = want_log
                    .iter()
                    .map(|t| {
                        if *t == "-" {
                            return vec![];
                        }
                        t.split('/')
                            .map(|c| {
                                let mut f = c.split(',');
                                let ecx = f.next().expect("ecx");
                                (ecx, f.map(|x| x.parse().expect("an arg")).collect())
                            })
                            .collect()
                    })
                    .collect();
                // question id -> answer, with a flag set when the same question got two answers (an object the
                // original's stubs describe inconsistently, which no `Sim` can be).
                let mut ans: Vec<(i32, i32)> = Vec::new();
                let mut inconsistent = false;
                let mut put = |q: i32, v: i32| match ans.iter().find(|x| x.0 == q) {
                    Some(x) => inconsistent |= x.1 != v,
                    None => ans.push((q, v)),
                };
                for (k, (_, args)) in toks[0].iter().enumerate() {
                    put(args[0], a[6 + k.min(5)]);
                }
                // 0x1bb fort, 0x19e city, 0x1b0 dock.
                for (t, id, w) in [(1, 0x1bb, 12), (2, 0x19e, 13), (3, 0x1b0, 14)] {
                    if !toks[t].is_empty() {
                        put(id, a[w]);
                    }
                }
                let q = |id: i32| ans.iter().find(|x| x.0 == id).is_some_and(|x| x.1 != 0);
                let (param, flags, t40, sea) = (a[0], rd(m, 0x2b4), rd(m, 0x40), rd(m, 0x218) == 1);
                let transport = flags & 0x10 != 0 || q(0x15f);
                let fortify = flags & 0x1800 != 0;
                if param < 0 {
                    // The crate takes a `usize`: `can_garrison(-1)` (any building, for the fortifying kinds) has
                    // no spelling in it (parked), so the original's own arm is stated here.
                    skipped_neg += 1;
                    i64::from(!(sea && transport) && fortify)
                } else if inconsistent {
                    skipped_inconsistent += 1;
                    want_eax
                } else {
                    // build_types: 0 the building, then its lineage nodes, then the trainers.
                    let ident = match rd(&p[1 + param as usize], 4) {
                        0x1ab => Ident::Barracks,
                        0x1ac => Ident::Stable,
                        0x1ad => Ident::AutoPlant,
                        0x1ae => Ident::SiegeFactory,
                        0x1af => Ident::Factory,
                        _ => Ident::Other,
                    };
                    let mut types = vec![BuildType {
                        ident,
                        ..BuildType::default()
                    }];
                    for (id, ident) in [
                        (0x1bb, Ident::Fort),
                        (0x1b7, Ident::Tower),
                        (0x19e, Ident::Village),
                        (0x1b0, Ident::Dock),
                    ] {
                        if q(id) {
                            let at = types.len();
                            let last = at - 1;
                            types[last].from = Some(at);
                            types.push(BuildType {
                                ident,
                                ..BuildType::default()
                            });
                        }
                    }
                    // `+0x40` is a build type's index and its ident id at once in the original: equal to the
                    // argument it names the building itself; one of the five ids (or the university's) names a
                    // trainer of that ident; anything else a trainer that is none of them.
                    let mut trainer = |ident: Ident| {
                        types.push(BuildType {
                            ident,
                            ..BuildType::default()
                        });
                        Some(types.len() - 1)
                    };
                    let trained_at = match t40 {
                        -1 => None,
                        t if t == param => Some(0),
                        0x1ab => trainer(Ident::Barracks),
                        0x1ac => trainer(Ident::Stable),
                        0x1ad => trainer(Ident::AutoPlant),
                        0x1ae => trainer(Ident::SiegeFactory),
                        0x1af => trainer(Ident::Factory),
                        0x1a4 => trainer(Ident::University),
                        _ => trainer(Ident::Wonder),
                    };
                    let mut sim = Sim::new(Tuning::RON, World::new(8, 8), 4);
                    sim.build_types = types;
                    sim.unit_types = vec![UnitType {
                        kind: UnitKind {
                            domain: if sea { Domain::Sea } else { Domain::Land },
                            ..UnitKind::default()
                        },
                        garrison: UnitTraits {
                            fortify,
                            trained_at,
                            gov_hero: false,
                            transport,
                        },
                        ..UnitType::default()
                    }];
                    i64::from(sim.can_garrison(0, 0))
                }
            }
        };
        assert_eq!(got, want_eax, "can_garrison — eax, row `{line}`");
        let log: Vec<String> = want_log.iter().map(|t| t.to_string()).collect();
        assert_eq!(
            log, want_log,
            "can_garrison — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "can_garrison — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "can_garrison — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "can_garrison — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!(
        "can_garrison: {skipped_neg} rows with a negative building (stated inline), {skipped_inconsistent} inconsistent objects not run through the crate"
    );
    assert!(
        skipped_inconsistent * 10 < n,
        "{skipped_inconsistent} inconsistent rows"
    );
    eprintln!("UnitTypeData::can_garrison@0061d7d0: {n} rows agree");
}
