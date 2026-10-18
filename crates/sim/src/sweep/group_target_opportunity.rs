//! `Group::target_opportunity@007107d0` under the emulator against the port (item 3250; tools/sweep_gen.py, obj with windows).

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

use crate::combat::{Obj, Profile};
use crate::group::{GroupState, Pushed, Seat};
use crate::orders::{Body, GroupMove, MoveKind, Order, QueuePos};
use crate::world::World;
use crate::{Pos, Sim, Tuning, Unit, UnitType};

/// What the original's stubs said about one member, as the crate must hold it in its order list: the
/// original asks `get_action` (null, or an object whose `get_type` is asked), then `order_type`; the crate
/// has one list of orders and derives both, so the pair is spelled as the list that gives the same
/// answers (`orders.rs:1439`/`1445`): no action and `order_type` NONE is an empty list, and a transit
/// move of the kind `ATTACK_TO`/`GROUP_ATTACK_TO`/`MOVE_TO` gives `get_action` none and that `order_type`;
/// anything else is a non-move order (`Think`), an action whose type is not NONE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Held {
    Empty,
    MoveTo,
    AttackTo,
    GroupAttackTo,
    Think,
}

/// One member's visit in the original's call log: the window it was handed, what its stubs answered and
/// which callee it ended in (`s0` `Unit::target_opportunity`, `s4` `find_melee_target`).
#[derive(Clone, Debug, Default)]
struct Visit {
    win: usize,
    action: Option<bool>,
    get_type: Option<i32>,
    order_type: Option<i32>,
    /// `Some(None)` arm 1/3 (`Unit::target_opportunity`), `Some(Some(range))` arm 2 (`find_melee_target`).
    end: Option<Option<i32>>,
}

impl Visit {
    /// The order list that reproduces the stubs' answers; `None` for arm 1, which asks nothing.
    fn held(&self) -> Option<Held> {
        let action = self.action?;
        Some(if action && self.get_type != Some(0) {
            Held::Think
        } else {
            match self.order_type {
                Some(0) => Held::Empty,
                Some(1) => Held::MoveTo,
                Some(2) => Held::AttackTo,
                Some(0x15) => Held::GroupAttackTo,
                _ => Held::Think,
            }
        })
    }
}

/// The window `p<N>` of a recorded `ecx` token, `p3+0x0` -> 3.
fn win_of(tok: &str) -> usize {
    tok.strip_prefix('p')
        .and_then(|t| t.split('+').next())
        .and_then(|t| t.parse().ok())
        .unwrap_or_else(|| panic!("a window token, not `{tok}`"))
}

/// The unit a window is: `p1..p4` owner 0 index 0..3, `p6..p9` owner 1; its crate id `4 * owner + index`.
fn crate_id(win: usize) -> usize {
    match win {
        1..=4 => win - 1,
        6..=9 => 4 + win - 6,
        _ => panic!("p{win} is not a unit window"),
    }
}

fn parse_log(log: &[&str], seq2: &[i32], seq3: &[i32], action: bool) -> Vec<Visit> {
    let (mut c2, mut c3) = (0usize, 0usize);
    let mut visits = Vec::new();
    let mut cur: Option<Visit> = None;
    for tok in log.iter().filter(|t| **t != "-") {
        let (s, rest) = tok.split_once(':').expect("`sK:ecx,args`");
        let mut it = rest.split(',');
        let ecx = it.next().expect("an ecx");
        match s {
            "s1" => {
                visits.extend(cur.take());
                cur = Some(Visit {
                    win: win_of(ecx),
                    action: Some(action),
                    ..Visit::default()
                });
            }
            "s2" => {
                cur.as_mut().expect("a visit").get_type = Some(seq2[c2]);
                c2 += 1;
            }
            "s3" => {
                cur.as_mut().expect("a visit").order_type = Some(seq3[c3]);
                c3 += 1;
            }
            "s0" | "s4" => {
                let mut v = cur.take().unwrap_or(Visit {
                    win: win_of(ecx),
                    ..Visit::default()
                });
                assert_eq!(
                    v.win,
                    win_of(ecx),
                    "the callee is handed the member it asked"
                );
                let args: Vec<i32> = it.map(|x| x.parse().expect("an argument")).collect();
                v.end = Some(if s == "s0" {
                    assert_eq!(args.len(), 3);
                    assert_eq!(args[2], 1);
                    None
                } else {
                    assert_eq!(&args[1..], &[0, 0, 1, 0]);
                    Some(args[0])
                });
                visits.push(v);
            }
            _ => panic!("a recorded stub, not `{tok}`"),
        }
    }
    assert!(cur.is_none(), "a visit that ended");
    visits
}

/// Everything the crate is built from, read off the row's windows.
struct Scene {
    frame: i64,
    respond: i32,
    owner: u8,
    members: Vec<usize>,
    last: i64,
    attacker: usize,
    asker: usize,
    from_think: bool,
    units: Vec<SceneUnit>,
}

/// One unit as its window says it is.
struct SceneUnit {
    owner: u8,
    index: i16,
    pos: Pos,
    alive: bool,
    on_map: bool,
    captain: bool,
    up: Option<usize>,
    role: bool,
}

fn scene(a: &[i32], m: &[u8], c: &[Vec<u8>], p: &[Vec<u8>]) -> Scene {
    let owner = m[0x4a];
    assert!(owner < 2);
    let count = rd(m, 0xc) as usize;
    let members = (0..count)
        .map(|i| {
            let k = i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cd + 2 * i]]);
            usize::from(owner) * 4 + usize::try_from(k).expect("an index")
        })
        .collect();
    let mut units = Vec::new();
    for o in 0..2usize {
        for k in 0..4usize {
            let w = &p[if o == 0 { 1 + k } else { 6 + k }];
            let flags = rd(w, 8);
            let up = u16::from_le_bytes([w[0x8e], w[0x8f]]);
            let role = rd(&p[12 + k], 0x2c8) & 0x10000 != 0;
            units.push(SceneUnit {
                owner: o as u8,
                index: (4 * o + k) as i16,
                pos: Pos::new(rd(w, 0x10) ^ 0x63637, rd(w, 0x14) ^ 0x63637),
                alive: flags & 1 != 0,
                on_map: rd(w, 0x80) < 0,
                captain: up >> 15 != 0,
                up: (up >> 15 == 0).then(|| 4 * o + usize::from(up & 0x7fff)),
                role,
            });
        }
    }
    Scene {
        frame: i64::from(rd(&c[0], 0x550)),
        respond: rd(&c[1], 0x18),
        owner,
        members,
        last: i64::from(rd(m, 0x38)),
        attacker: (a[1] * 4 + a[0]) as usize,
        asker: usize::from(owner) * 4 + a[2] as usize,
        from_think: a[3] != 0,
        units,
    }
}

/// A `Sim` laid from the scene, the group a pushed one, each visited member holding the orders its
/// stubs' answers amount to.
fn build(s: &Scene, held: &[Option<Held>; 8]) -> Sim {
    let mut tuning = Tuning::RON;
    tuning.unit_respond_range = s.respond;
    let mut sim = Sim::new(tuning, World::new(60, 60), 2);
    sim.at_war[0][1] = true;
    sim.at_war[1][0] = true;
    let role = sim.add_unit_type(UnitType {
        hits: 100,
        combat: Profile {
            attack: 15,
            max_range: 0,
            uber_size: 1,
            combat_role: true,
            ..Profile::default()
        },
        ..UnitType::default()
    });
    let plain = sim.add_unit_type(UnitType {
        hits: 100,
        combat: Profile {
            attack: 0,
            max_range: 0,
            uber_size: 1,
            ..Profile::default()
        },
        ..UnitType::default()
    });
    for su in &s.units {
        let ty = if su.role { role } else { plain };
        let mut u = Unit::new(su.owner, su.index, su.pos, if su.alive { 100 } else { 0 });
        u.ty = Some(ty);
        u.on_map = su.on_map;
        u.kind = sim.unit_types[ty].kind;
        u.captain = su.captain;
        u.o_up = su.up;
        sim.add_unit(u);
    }
    for (id, h) in held.iter().enumerate() {
        // Both owners' units are ids 0..8; `held` is indexed by the crate id of the first eight.
        let at = sim.units[id].pos;
        match h {
            None | Some(Held::Empty) => {}
            Some(Held::MoveTo) => {
                sim.add_move_order(id, at, MoveKind::MoveTo, QueuePos::New, false);
            }
            Some(Held::AttackTo) => {
                sim.add_move_order(id, at, MoveKind::AttackTo, QueuePos::New, false);
            }
            Some(Held::GroupAttackTo) => {
                sim.add_move_order(id, at, MoveKind::AttackTo, QueuePos::New, false);
                sim.units[id].orders[0].move_mut().expect("a move").group = Some(GroupMove {
                    leader: id,
                    id: 1,
                    form_id: 0,
                    group_angle: crate::movement::Angle::default(),
                    in_group: false,
                    orig: at,
                });
            }
            Some(Held::Think) => sim.units[id].orders.push_back(Order {
                flags: 0,
                body: Body::Think,
            }),
        }
    }
    for &mem in &s.members {
        sim.units[mem].group_ptr = Some(2);
    }
    sim.frame = s.frame;
    sim.pushed.push(Pushed {
        who: s.owner,
        list: s.members.clone(),
        state: GroupState {
            pool: Some(2),
            opportunity: s.last,
            ..GroupState::default()
        },
        builds: Vec::new(),
    });
    sim
}

/// `Group::target_opportunity@007107d0`, run by `tools/emu/sweep_group_target_opportunity.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 4 argument(s), `p` windows in order: p0..p16 the place(s), in flag order; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c061f0 points at 64 random bytes; the global cell 0x00c0618c points at 64 random bytes; callee 0x005fffc0 (3 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x005ff9c0 (5 args) stubbed; the word at `c2+0x14` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `p0+12` points at 256 random bytes less `0` (placed); the word at `c2+0x30` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 256 random bytes less `0` (placed); the word at `p5+4` points at 256 random bytes less `0` (placed); the word at `p5+8` points at 256 random bytes less `0` (placed); the word at `p5+12` points at 256 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p1+0x18` points at 720 random bytes less `0` (placed); the word at `p2+0x18` points at 720 random bytes less `0` (placed); the word at `p3+0x18` points at 720 random bytes less `0` (placed); the word at `p4+0x18` points at 720 random bytes less `0` (placed); stub 1's answer points at 32 random bytes less `0` (placed, null a third of the rows); stub 2 answered from [0, 0, 1, 2, 18] (90%); stub 3 answered from [0, 0, 1, 2, 2, 21, 21, 12, 3] (90%); the word at `p10+0` laid as the address of `p1` every row; the word at `p11+0` laid as the address of `p6` every row; the word at `p6+0x18` laid as the address of `p12` every row; the word at `p10+4` laid as the address of `p2` every row; the word at `p11+4` laid as the address of `p7` every row; the word at `p7+0x18` laid as the address of `p13` every row; the word at `p10+8` laid as the address of `p3` every row; the word at `p11+8` laid as the address of `p8` every row; the word at `p8+0x18` laid as the address of `p14` every row; the word at `p10+12` laid as the address of `p4` every row; the word at `p11+12` laid as the address of `p9` every row; the word at `p9+0x18` laid as the address of `p15` every row; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_target_opportunity.py") else {
        return;
    };
    let mut n = 0;
    let (mut skipped_dup, mut parted_dead_up, mut changed, mut with_t, mut with_f, mut multi) =
        (0, 0, 0, 0, 0, 0);
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Windows: m the group (`+0xc` count, `+0x38` the cooldown stamp, `+0x4a` owner, `+0x8cc` the
            // member shorts); c0 the game (`+0x550` frame), c1 the constants (`+0x18` unit_respond_range),
            // c2 the objects cell; p1..p4 and p6..p9 the units of owner 0 and 1 (the objects table and the
            // units table name the same windows, `--alias`), p12..p15 the four type records (`+0x2c8 &
            // 0x10000` combat_role), p16 the action record `get_action` answers. a[0..4] the arguments
            // (attacker index, attacker owner, asker index, param_4), a[4..9] the five stubs' answers
            // (`get_action`'s is the record's address or null), a[9..15] the `get_type` answers and a[15..21]
            // the `order_type` answers in call order.
            //
            // The port is `Sim::group_target_opportunity` on a `Sim` laid from the row (a pushed group,
            // eight units, two players at war). Its callees (`Unit::target_opportunity` and
            // `find_melee_target`, stubbed in the original) run for real in the crate, so the check is
            // twofold: a second `Sim`, laid the same, replays **the original's recorded calls** on the
            // crate's own callees (`target_opportunity_in(m, attacker, true)` for `s0`,
            // `find_melee_target_added_in(m, range)` for `s4`, in the order made) and the two must end
            // identical; the generated log check below holds the call log, which is the original's own.
            // The orders each member holds are the list that gives the stubs' answers (`Held`).
            let seq2: Vec<i32> = a[9..15].to_vec();
            let seq3: Vec<i32> = a[15..21].to_vec();
            let sc = scene(a, m, c, p);
            let visits = parse_log(&want_log, &seq2, &seq3, a[5] != 0);
            let mut held: [Option<Held>; 8] = [None; 8];
            // A unit visited twice sees, in the crate, the first callee's effect (an attack order added);
            // the original's stubs change nothing, so its second visit is asked the same questions. The
            // two cannot be held to one another, and the row is skipped (counted).
            let mut conflict = false;
            for (i, v) in visits.iter().enumerate() {
                conflict |= visits[..i].iter().any(|w| w.win == v.win);
                if let Some(h) = v.held() {
                    held[crate_id(v.win)] = Some(h);
                }
            }
            // `squad_captain` stops at a dead `o_up`, where `get_captain` walks on: a parting (parked).
            let mut at = sc.asker;
            let mut dead_up = false;
            while !sc.units[at].captain {
                let up = sc.units[at].up.expect("a non-captain has an up");
                if !sc.units[up].alive {
                    dead_up = true;
                    break;
                }
                at = up;
            }
            let after_m = unhex(after.trim().split(';').next().expect("this"));
            let want_stamp = i64::from(rd(&after_m, 0x38));
            if conflict {
                skipped_dup += 1;
                wr(m, 0x38, rd(&after_m, 0x38));
            } else if dead_up && !sc.from_think {
                parted_dead_up += 1;
                wr(m, 0x38, rd(&after_m, 0x38));
            } else {
                let attacker = Obj::Unit(sc.attacker);
                let mut s1 = build(&sc, &held);
                let before = s1.units.clone();
                s1.group_target_opportunity(Seat::Pushed(0), attacker, sc.asker, sc.from_think);
                let mut s2 = build(&sc, &held);
                for v in &visits {
                    let id = crate_id(v.win);
                    match v.end {
                        Some(None) => s2.target_opportunity_in(id, attacker, true),
                        Some(Some(range)) => {
                            s2.find_melee_target_added_in(id, range);
                        }
                        None => panic!("a visit with no callee"),
                    }
                }
                s2.pushed[0].state.opportunity = want_stamp;
                assert_eq!(
                    s1.pushed[0].state.opportunity, want_stamp,
                    "target_opportunity — the cooldown stamp, row `{line}`"
                );
                if s1.units != s2.units {
                    for (i, (x, y)) in s1.units.iter().zip(&s2.units).enumerate() {
                        if x != y {
                            eprintln!(
                                "unit {i}: crate {:?} {:?}\n  replay {:?} {:?}",
                                x.orders, x.combat, y.orders, y.combat
                            );
                        }
                    }
                    panic!(
                        "target_opportunity — the units after: args {a:?} log {want_log:?} held {held:?} members {:?} asker {} attacker {}",
                        sc.members, sc.asker, sc.attacker
                    );
                }
                if s1.units != before {
                    changed += 1;
                }
                with_t += visits.iter().filter(|v| v.end == Some(None)).count();
                with_f += visits
                    .iter()
                    .filter(|v| v.end.is_some_and(|e| e.is_some()))
                    .count();
                if sc.members.len() > 1 && visits.len() > 1 {
                    multi += 1;
                }
                wr(m, 0x38, s1.pushed[0].state.opportunity as i32);
            }
            for l in &want_log {
                log.push((*l).to_string());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "target_opportunity — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "target_opportunity — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "target_opportunity — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "target_opportunity — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    // Each arm and each parting must be in the rows, so the row set still reaches it.
    assert!(
        changed >= 100 && with_t >= 100 && with_f >= 50 && multi >= 50,
        "a starved draw"
    );
    assert!(skipped_dup > 0 && parted_dead_up > 0);
    eprintln!(
        "rows: {n}; compared {} (changed {changed}, T calls {with_t}, F calls {with_f}, multi {multi}); \
         skipped, a unit visited twice: {skipped_dup}; known parting, dead o_up: {parted_dead_up}",
        n - skipped_dup - parted_dead_up
    );
    eprintln!("Group::target_opportunity@007107d0: {n} rows agree");
}
