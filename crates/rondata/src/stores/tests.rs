//! The frozen scan's tests (item 4159, `docs/DECISIONS.md` entry 131 §1).
//!
//! The install-backed ones run the scan once over `riseofnations.exe` and
//! pin, field by field, every store it finds through a leader pointer, every
//! store at the displacement it cannot tell apart, and every untyped call a
//! leader pointer reaches the field through. A new writer, a lost one, or a
//! site the scan stops being able to classify turns them red.
//!
//! Offsets are `LeaderData`'s (`rise.pdb`, `types.txt`); `docs/LEADERS.md`
//! cites each in the section that describes the leaf.

use super::*;
use crate::testenv::install_root;
use std::sync::OnceLock;

const fn field(name: &'static str, lo: u32, hi: u32) -> Field {
    Field { name, lo, hi }
}

/// The live field the scan is seen red on: `dow`, `int[8]` (`docs/LEADERS.md`
/// §7.1), which `Leader::action_declare` counts (item 4159's VII-b1).
const DOW: Field = field("dow", 0x270, 0x290);
/// The same scan at the wrong displacement: `invaders`, the next `int[8]`.
const WRONG: Field = field("invaders", 0x290, 0x2b0);

/// The three leaves this step releases as frozen (`docs/LEADERS.md` §7.1,
/// §18.1).
const FROZEN: &[Field] = &[
    field("score_explored", 0x1c, 0x20),
    field("score_pop", 0x34, 0x38),
    field("last_spoke", 0x2f4, 0x314),
];

/// The earlier quiet releases, run through the same scan: item 4162's six
/// (`docs/CHECKSUM.md` §20) and §20's never-reached constants.
const QUIET: &[Field] = &[
    field("cannon_time", 0x48, 0x4c),
    field("handicap", 0x4c, 0x50),
    field("multi_diff", 0x50, 0x54),
    field("city_name", 0x40c, 0x410),
    field("misery", 0x7ec, 0x7f0),
    field("anti_att", 0x7f4, 0x7f8),
    field("ctw_hero_retreat_stamp", 0x204, 0x208),
    field("village_mine", 0x404, 0x408),
    field("base_rate", 0x4b0, 0x4c8),
    field("support_stamp", 0x7b0, 0x7b4),
    field("take_att_disabled", 0x7fc, 0x800),
    field("neutral_attrition", 0x800, 0x804),
    field("disable_building_attrition", 0x804, 0x808),
    field("goody_box_techs", 0x868, 0x86c),
    field("goody_box_units", 0x870, 0x874),
    field("supported", 0x88c, 0x8a4),
    field("trade_profit", 0x8ec, 0x8f0),
    field("move_orders_issued", 0x934, 0x938),
    field("column_orders_issued", 0x938, 0x93c),
    field("best_pop", 0x99c, 0x9a0),
    field("best_attack", 0x9a0, 0x9a4),
    field("best_armor", 0x9a4, 0x9a8),
    field("best_move", 0x9a8, 0x9ac),
    field("nukes_in_flight", 0xa3c, 0xa40),
];

fn all_fields() -> Vec<Field> {
    let mut v = vec![DOW, WRONG];
    v.extend_from_slice(FROZEN);
    v.extend_from_slice(QUIET);
    v
}

/// The install and the listing tool, or a line saying which is missing.
fn tools() -> Option<(String, String)> {
    let Some(root) = install_root() else {
        eprintln!("skipping: no install (set RON_INSTALL)");
        return None;
    };
    let Some(od) = objdump() else {
        eprintln!("skipping: no llvm-objdump (set LLVM_OBJDUMP)");
        return None;
    };
    Some((root, od))
}

/// One scan of every field, shared by the tests of this binary.
fn scanned() -> Option<&'static (Scan, Image)> {
    static SCAN: OnceLock<Option<(Scan, Image)>> = OnceLock::new();
    SCAN.get_or_init(|| {
        let (root, od) = tools()?;
        Some(scan_install(&root, &od, &all_fields(), true).expect("the scan runs"))
    })
    .as_ref()
}

/// A pinned store: `(instruction, function, the constant stored)`, `None`
/// for a live store.
type Pin = (u32, u32, Option<u32>);

fn direct(s: &Scan, f: &str) -> Vec<Pin> {
    let mut v: Vec<Pin> = s.fields[f]
        .direct
        .iter()
        .map(|x| {
            let value = match x.value {
                Stored::Const(c) => Some(c),
                Stored::Live => None,
            };
            (x.at, x.func, value)
        })
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// The unresolved stores: `(instruction, function, reached)`.
fn unresolved(s: &Scan, f: &str) -> Vec<(u32, u32, bool)> {
    let mut v: Vec<_> = s.fields[f]
        .unresolved
        .iter()
        .map(|x| (x.at, x.func, x.reached))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Whether a callee is untyped: indirect, undecorated (a C function such as
/// `_memset`), or taking a `void *` (`PAX`, `PBX`: the vector constructor
/// and destructor iterators). A typed callee handed an interior pointer
/// writes inside the record type it is typed to, which no scalar field of
/// `LeaderData` is.
fn untyped(s: &Scan, callee: Option<u32>) -> bool {
    let Some(c) = callee else {
        return true;
    };
    let names = s.names.get(&c).map(Vec::as_slice).unwrap_or(&[]);
    !names.iter().any(|n| n.starts_with('?'))
        || names.iter().any(|n| n.contains("PAX") || n.contains("PBX"))
}

/// One untyped call a leader pointer reaches a field through:
/// `(call, function, callee)`.
type Bulk = (u32, u32, String);

/// The calls that hand an untyped callee a leader pointer whose reach meets
/// `f`. A call handed two pointers (`DataWalk::walk(begin, end)`) reaches
/// `[lowest, highest)`; one handed one reaches everything above it.
fn bulk(s: &Scan, f: &Field) -> Vec<Bulk> {
    let mut calls: BTreeMap<(u32, u32), (EscapeKind, Option<u32>, Vec<u32>)> = BTreeMap::new();
    for e in &s.escapes {
        let untyped = match e.kind {
            EscapeKind::Push => untyped(s, e.callee),
            EscapeKind::String | EscapeKind::Heap => true,
            EscapeKind::This => false,
        };
        if untyped {
            let c = calls
                .entry((e.via, e.func))
                .or_insert((e.kind, e.callee, Vec::new()));
            c.2.push(e.off);
        }
    }
    calls
        .into_iter()
        .filter_map(|((via, func), (kind, callee, offs))| {
            let lo = *offs.iter().min()?;
            let hi = offs.iter().max().copied().filter(|&h| h > lo);
            let reaches = lo <= f.lo && hi.is_none_or(|h| f.lo < h);
            reaches.then(|| {
                let name = match (kind, callee) {
                    (EscapeKind::String, _) => "(string store)".to_owned(),
                    (EscapeKind::Heap, _) => "(heap store)".to_owned(),
                    (_, None) => "(indirect)".to_owned(),
                    (_, Some(c)) => s.name(c).to_owned(),
                };
                (via, func, name)
            })
        })
        .collect()
}

fn pins(rows: &[(u32, u32, &str)]) -> Vec<Bulk> {
    rows.iter().map(|&(a, f, n)| (a, f, n.to_owned())).collect()
}

/// **Red first, on a field with live writers.** `dow` is counted by
/// `Leader::action_declare (006dab50)`'s two increments of
/// `dow[me][t]` (`inc [esi + 4*edi + 0x270]`, `esi` its `this`) and cleared
/// by `Leader::init (006e3930)`'s per-slot loop. The scan finds all three.
/// At the wrong displacement (`invaders`, the next array) it finds neither
/// increment; and with the leader methods' `this` left unknown it finds
/// neither as a leader's store: the provenance is what tells them apart.
#[test]
fn the_scan_finds_dow_s_writers_and_a_perturbed_scan_misses_them() {
    let Some((s, _)) = scanned() else {
        return;
    };
    let declare: &[Pin] = &[(0x6dad3e, 0x6dab50, None), (0x6dad61, 0x6dab50, None)];
    let found = direct(s, DOW.name);
    assert_eq!(
        found,
        [declare, &[(0x6e3d44, 0x6e3930, Some(0))]].concat(),
        "dow's writers"
    );
    assert_eq!(unresolved(s, DOW.name), [], "dow: nothing left unclassified");

    let wrong = direct(s, WRONG.name);
    assert!(
        !wrong.iter().any(|p| declare.contains(p)),
        "the wrong displacement finds action_declare's increments: {wrong:x?}"
    );

    let (root, od) = tools().expect("the install was there for the first scan");
    let (blind, _) = scan_install(&root, &od, &[DOW], false).expect("the perturbed scan runs");
    let blind_direct = direct(&blind, DOW.name);
    assert!(
        !blind_direct.iter().any(|p| declare.contains(p)),
        "with `this` unknown the increments are still a leader's: {blind_direct:x?}"
    );
}

/// The untyped calls that reach every frozen leaf: the whole-record clears
/// (`_memset` in `Leader::close (006b7f80)`, `Leader::init (006e3930)` and
/// `Leaders::clear (006ed5d0)`, each storing 0), the save game's walk of
/// `[+0x8, +0x692a)` in `LeaderData::walk_data (006d6750)`, which restores
/// what a writer stored, and `leaders`' constructor and destructor
/// iterators (the destructor's twice: at exit, and in the constructor's
/// unwind funclet), which run `Leader::Leader` and `~Leader` (each scanned
/// in its own right, and neither writing these leaves).
const BULK_TO_FROZEN: &[(u32, u32, &str)] = &[
    (0x6b7f8c, 0x6b7f80, "_memset"),
    (0x6d6796, 0x6d6750, "(indirect)"),
    (0x6e3995, 0x6e3930, "_memset"),
    (0x6ed5ee, 0x6ed5d0, "_memset"),
    (0x6ed936, 0x6ed900, "??_N@YGXPAXIIP6EX0@Z1@Z"),
    (0xa592c1, 0xa592b0, "??_M@YGXPAXIIP6EX0@Z@Z"),
    (0xab5551, 0xab5510, "??_M@YGXPAXIIP6EX0@Z@Z"),
];

/// **The frozen leaves** (entry 131 §1's second way out): every store the
/// executable makes to each writes the constant 0 our walk holds.
///
/// - `score_explored` (+0x1c): `Leader::compute_explore_score (006bc5e0)`,
///   `Leader::reset_score (006e37f0)`, and `Leader::compute_score
///   (006ec560)` twice (its head and its Armageddon arm).
/// - `score_pop` (+0x34): `Leader::compute_pop_score (006bc190)`,
///   `reset_score`, `compute_score` twice.
/// - `last_spoke` (+0x2f4): `Leader::init`'s per-slot clear alone.
///
/// The one unresolved store at each displacement is reached by no path:
/// `Leader::diplomacy (006bc950)`'s jump-table bytes and `Console::
/// exec_keymap (007c2ce0)`'s, decoded as code.
#[test]
fn the_frozen_leaves_every_store_writes_zero() {
    let Some((s, _)) = scanned() else {
        return;
    };
    let writers: [(&str, &[Pin]); 3] = [
        (
            "score_explored",
            &[
                (0x6bc5e0, 0x6bc5e0, Some(0)),
                (0x6e37fc, 0x6e37f0, Some(0)),
                (0x6ec5a0, 0x6ec560, Some(0)),
                (0x6ec636, 0x6ec560, Some(0)),
            ],
        ),
        (
            "score_pop",
            &[
                (0x6bc190, 0x6bc190, Some(0)),
                (0x6e380e, 0x6e37f0, Some(0)),
                (0x6ec5ca, 0x6ec560, Some(0)),
                (0x6ec659, 0x6ec560, Some(0)),
            ],
        ),
        ("last_spoke", &[(0x6e3d6f, 0x6e3930, Some(0))]),
    ];
    let unreached: [(&str, &[(u32, u32, bool)]); 3] = [
        ("score_explored", &[(0x6c1741, 0x6bc950, false)]),
        (
            "score_pop",
            &[(0x7c3783, 0x7c2ce0, false), (0x7c3797, 0x7c2ce0, false)],
        ),
        ("last_spoke", &[]),
    ];
    for ((name, pinned), (_, garbage)) in writers.iter().zip(&unreached) {
        let found = direct(s, name);
        assert_eq!(&found, pinned, "{name}'s writers");
        assert!(
            found.iter().all(|p| p.2 == Some(0)),
            "{name}: every writer stores 0"
        );
        assert_eq!(&unresolved(s, name), garbage, "{name}: unclassified stores");
        let f = FROZEN.iter().find(|f| f.name == *name).expect("a frozen field");
        assert_eq!(bulk(s, f), pins(BULK_TO_FROZEN), "{name}: untyped calls");
    }
}

/// **The earlier quiet releases**, each writer set pinned as the scan finds
/// it. None contradicts the document that released it: `multi_diff`
/// (`docs/LEADERS.md` §16.2) and `city_name` (§1.2) match their writer lists
/// store for store; `misery`'s four stores all write 0 (§12.3);
/// `cannon_time`'s are the seed and `TurnControl::start_cannon_time
/// (00956500)` (§16.2); `handicap`'s `Game::init_handicaps (0058abf0)` and
/// the scenario loader; `anti_att`'s `Leader::calc_anti_attrition
/// (006cdcc0)` and the seed. Every never-reached constant of
/// `docs/CHECKSUM.md` §20 is written by `Leader::init`, `reset_score` or the
/// scenario layer (`ScenarioFuncSet`) alone.
#[test]
fn the_quiet_releases_writers_are_the_documented_ones() {
    let Some((s, _)) = scanned() else {
        return;
    };
    let init = 0x6e3930;
    let reset = 0x6e37f0;
    let writers: &[(&str, &[Pin])] = &[
        ("cannon_time", &[(0x6e3af3, init, None), (0x9565eb, 0x956500, None)]),
        (
            "handicap",
            &[
                (0x58ade8, 0x58abf0, None),
                (0x58ae07, 0x58abf0, None),
                (0x9ac1fc, 0x9ac080, None),
                (0x9ac457, 0x9ac080, None),
                (0x9ac643, 0x9ac080, None),
            ],
        ),
        (
            "multi_diff",
            &[
                (0x58ae49, 0x58abf0, None),
                (0x6d2474, 0x6d2470, None),
                (0x6d2c24, 0x6d2bf0, Some(2)),
                (0x95967a, 0x959500, Some(3)),
                (0x9898bc, 0x9897d0, None),
                (0x99bc0d, 0x99bc00, None),
                (0x9ac5aa, 0x9ac080, None),
                (0x9e5386, 0x9e5350, None),
            ],
        ),
        (
            "city_name",
            &[
                (0x6e3b14, init, Some(0)),
                (0x73614a, 0x735c90, None),
                (0x7362d2, 0x735c90, Some(1)),
            ],
        ),
        (
            "misery",
            &[
                (0x6cdff0, 0x6cdff0, Some(0)),
                (0x6dc49d, 0x6dc490, Some(0)),
                (0x6de007, 0x6dcb60, Some(0)),
                (0x6e497e, init, Some(0)),
            ],
        ),
        (
            "anti_att",
            &[
                (0x6cdcce, 0x6cdcc0, Some(0x4380_0000)),
                (0x6cdcda, 0x6cdcc0, Some(0)),
                (0x6cdd5f, 0x6cdcc0, Some(0)),
                (0x6cdd8d, 0x6cdcc0, None),
                (0x6cdde8, 0x6cdcc0, None),
                (0x6cde3c, 0x6cdcc0, None),
                (0x6cde66, 0x6cdcc0, Some(0)),
                (0x6cde8e, 0x6cdcc0, None),
                (0x6e4955, init, Some(0)),
            ],
        ),
        ("ctw_hero_retreat_stamp", &[(0x6e3dd0, init, Some(0xffff_fc18))]),
        ("village_mine", &[(0x6e4011, init, Some(0))]),
        ("base_rate", &[(0x6e3f04, init, Some(0)), (0x9fbbf6, 0x9fbb80, None)]),
        ("support_stamp", &[(0x6e3baa, init, Some(0))]),
        (
            "take_att_disabled",
            &[
                (0x6e4990, init, Some(0)),
                (0xa028a6, 0xa02880, Some(0)),
                (0xa028e6, 0xa028c0, Some(1)),
            ],
        ),
        (
            "neutral_attrition",
            &[(0xa02a39, 0xa02a00, None), (0xa02a4a, 0xa02a00, Some(0))],
        ),
        (
            "disable_building_attrition",
            &[
                (0x6e4996, init, Some(0)),
                (0xa029a6, 0xa02980, Some(0)),
                (0xa029e6, 0xa029c0, Some(1)),
            ],
        ),
        ("goody_box_techs", &[(0x6e38a7, reset, Some(0))]),
        ("goody_box_units", &[(0x6e38ad, reset, Some(0))]),
        ("supported", &[(0x6e3f26, init, Some(0))]),
        ("trade_profit", &[(0x6e38b3, reset, Some(0))]),
        ("move_orders_issued", &[(0x6e38c5, reset, Some(0))]),
        ("column_orders_issued", &[(0x6e38cb, reset, Some(0))]),
        ("best_pop", &[(0x6e4a60, init, Some(0))]),
        ("best_attack", &[(0x6e4a66, init, Some(0))]),
        ("best_armor", &[(0x6e4a6c, init, Some(0))]),
        ("best_move", &[(0x6e4a72, init, Some(0))]),
        ("nukes_in_flight", &[(0x6e4aa2, init, Some(0))]),
    ];
    assert_eq!(writers.len(), QUIET.len(), "one pin a quiet field");
    for (name, pinned) in writers {
        assert_eq!(&direct(s, name), pinned, "{name}'s writers");
        // The reached unresolved stores, each told apart by hand: one, in
        // `IFaceAirGarrison::update_visibility (007fb320)`, whose `this`
        // (`helicopters_garrisoned`, +0x4b4, `types.txt`) it reloads from an
        // `esp` spill the pass does not track.
        let reached: Vec<_> = unresolved(s, name).into_iter().filter(|u| u.2).collect();
        let expected: &[(u32, u32, bool)] = if *name == "base_rate" {
            &[(0x7fb684, 0x7fb320, true)]
        } else {
            &[]
        };
        assert_eq!(reached, expected, "{name}: reached unclassified stores");
    }
}

/// The two references to `leaders` are the only static words the loader
/// rebases into `leaders.list`, each holding its base (so the scan's
/// provenance sees every static pointer); every function's fixpoint
/// converged; and only two names of the map return a leader.
#[test]
fn the_scan_s_premises_hold_on_the_image() {
    let Some((s, img)) = scanned() else {
        return;
    };
    assert_eq!(img.leaders, 0x00e3_a390, "`leaders`, the map's");
    let refs: Vec<u32> = img.refs.iter().copied().collect();
    assert_eq!(refs, [0x00ad_0530, 0x00c0_61e0], "the two references");
    assert_eq!(
        img.static_pointers,
        [(0x00ad_0530, img.leaders), (0x00c0_61e0, img.leaders)],
        "the static pointers into leaders.list"
    );
    assert_eq!(
        img.leader_returning,
        [
            "??ALeaders@@QBEABVLeader@@H@Z",
            "?get_scary_console_leader@GameAccessConst@@SAPAVLeader@@XZ"
        ],
        "the functions returning a leader"
    );
    assert!(s.unconverged.is_empty(), "unconverged: {:x?}", s.unconverged);
    assert!(
        !s.escapes.iter().any(|e| e.kind == EscapeKind::Heap),
        "a leader pointer stored to the heap"
    );
    assert!(s.functions > 50_000, "{} functions scanned", s.functions);
}

// ---------------------------------------------------------------------------
// Without the install: the decoration, the listing's syntax, and a small
// image the scan is seen red on.

#[test]
fn decorated_names_are_read() {
    assert_eq!(class_of("?compute_score@Leader@@QAEXH@Z"), Some("Leader"));
    assert_eq!(class_of("??0Leaders@@QAE@XZ"), Some("Leaders"));
    assert_eq!(class_of("??_GLeader@@QAEPAXI@Z"), Some("Leader"));
    assert_eq!(class_of("_memset"), None);
    assert!(is_thiscall("?compute_score@Leader@@QAEXH@Z"));
    assert!(is_thiscall("?get_diff@LeaderData@@QBEHXZ"));
    assert!(!is_thiscall(
        "?get_scary_console_leader@GameAccessConst@@SAPAVLeader@@XZ"
    ));
    assert!(returns_leader("??ALeaders@@QBEABVLeader@@H@Z"));
    assert!(returns_leader(
        "?get_scary_console_leader@GameAccessConst@@SAPAVLeader@@XZ"
    ));
    assert!(!returns_leader(
        "?issue_declare@CommandManager@@QAEXPBVLeaderOut@@HH@Z"
    ));
    assert!(mentions_leader(
        "?issue_declare@CommandManager@@QAEXPBVLeaderOut@@HH@Z"
    ));
}

#[test]
fn listing_lines_are_read() {
    let i = Insn::parse("  6dad3e:      \tinc\tdword ptr [esi + 4*edi + 0x270]").expect("a line");
    assert_eq!((i.at, i.op.as_str()), (0x6dad3e, "inc"));
    assert_eq!(
        i.operands(),
        [Opnd::Mem(Mem {
            base: Some(ESI),
            index: Some((EDI, 4)),
            disp: 0x270,
            seg: false,
            width: 4,
        })]
    );
    let r = Insn::parse("  4014cb:      \trep\t\tstosd\tdword ptr es:[edi], eax").expect("a line");
    assert_eq!(r.op, "rep stosd");
    assert!(is_string_store(&r.op, &r.args));
    let n = Insn::parse("  6ec5ee:      \tmov\tecx, dword ptr [ebp - 0x10]").expect("a line");
    match n.operands()[1] {
        Opnd::Mem(m) => assert_eq!(m.disp as i32, -0x10),
        o => panic!("{o:?}"),
    }
}

/// A three-function image: a leader method storing through its `this`, an
/// other class's method storing at the same displacement, and a free
/// function storing through a reference load and through `leaders.list`
/// indexed by a slot.
const MAP: &str = "\
 0001:00000000       ?f@Leader@@QAEXXZ          00401000 f   a.obj
 0001:00000100       ?g@Unit@@QAEXXZ            00401100 f   a.obj
 0001:00000200       ?h@@YAXH@Z                 00401200 f   a.obj
 0002:0000b530       ?leaders@GameAccess@@2AAVLeaders@@A 00ad0530     a.obj
 0003:00234390       ?leaders@@3VLeaders@@A     00e3a390     a.obj
";

const LISTING: &str = "  401000:      \tmov\tesi, ecx
  401002:      \tmov\tdword ptr [esi + 0x1c], 0x0
  401009:      \tret
  401100:      \tmov\tesi, ecx
  401102:      \tmov\tdword ptr [esi + 0x1c], 0x5
  401109:      \tret
  401200:      \tmov\teax, dword ptr [0xad0530]
  401205:      \tmov\tdword ptr [eax + 0x1c], 0x7
  40120c:      \timul\tecx, edx, 0x6eec
  401212:      \tinc\tdword ptr [ecx + 0xe3a3ac]
  401218:      \tret
";

fn small(seed_this: bool) -> Scan {
    let map = Map::parse(MAP);
    let words = |_: u32, _: usize| None;
    let ctx = Context {
        map: &map,
        leaders: 0x00e3_a390,
        refs: [0x00ad_0530].into_iter().collect(),
        words: &words,
        seed_this,
    };
    scan(
        &ctx,
        LISTING.lines().filter_map(Insn::parse),
        &[field("score_explored", 0x1c, 0x20)],
    )
}

#[test]
fn a_small_image_is_scanned_and_seen_red() {
    let s = small(true);
    assert_eq!(s.functions, 3);
    assert_eq!(
        direct(&s, "score_explored"),
        [
            (0x401002, 0x401000, Some(0)),
            (0x401205, 0x401200, Some(7)),
            (0x401212, 0x401200, None),
        ],
        "the leader method's store, the reference's and the indexed slot's; \
         not Unit::g's"
    );
    assert_eq!(unresolved(&s, "score_explored"), []);
    // With `this` unknown the leader method's store is no leader's.
    let blind = small(false);
    assert!(
        !direct(&blind, "score_explored")
            .iter()
            .any(|p| p.0 == 0x401002),
        "a blind scan still credits the leader method"
    );
}
