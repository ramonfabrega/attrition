//! **The second pair** (DECISIONS 53 §2, item 971): run54's and run53's games
//! with the lobby's difficulty at its top setting, Toughest, and nothing else
//! moved — run346 on East Indies and run347 on Great Lakes. The words are
//! [`SECOND_WORD_EAST_INDIES`] and [`SECOND_WORD_GREAT_LAKES`]; the first
//! pair's `LONG_WORD_*` stay as its closed floor. `docs/AI.md` §80 has the
//! lever, the arms as hypotheses, and what each word turned out to be.

use super::*;

use crate::diff::harness::{
    attributed_sites, debug_armies, debug_leader, debug_watch, site_window,
};
use crate::diff::testkit::*;
use crate::testenv::{dump, install};

/// A long capture of the second pair walked from its own start dump: the
/// frame the draw count first parts, the frame the label sequence first
/// parts, and the trace's last frame. `None` when the capture is not on
/// this machine.
pub(crate) struct Walk {
    pub count: i64,
    pub sequence: i64,
    pub last: i64,
    pub difficulty: i32,
    /// **The closing whole-map state, scored** (item 1099): a long trace
    /// that ends on the game's own end writes one, as run347 does on 5931
    /// when the human is defeated, and the first pair's
    /// [`crate::diff::endpoint::walk_to_close`] compares every unit,
    /// building and city on it. `None` when the dump has none.
    pub endpoint: Option<crate::diff::endpoint::EndpointResult>,
}

/// The two maps' setup differs in one borrow each, as run54's and run53's
/// own tests do: East Indies takes run38's start dump and the pasture from
/// its own trace, Great Lakes the sibling dumps.
pub(crate) fn walk_second(gamelog: &str, tracelog: &str, east_indies: bool) -> Option<Walk> {
    walk_second_probed(gamelog, tracelog, east_indies, &mut |_, _| {})
}

/// [`walk_second`], handing `probe` the simulation after each tick `f`
/// (item 1120: a packet's state read on its own frame).
pub(crate) fn walk_second_probed(
    gamelog: &str,
    tracelog: &str,
    east_indies: bool,
    probe: &mut dyn FnMut(i64, &crate::diff::setup::Built),
) -> Option<Walk> {
    let inst = install()?;
    let (Some(path), Some(trace)) = (dump(gamelog), trace(tracelog)) else {
        eprintln!("skipping: no {gamelog} (set RON_GAMELOG_DIR)");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().unwrap();
    let sib_texts: Vec<String> = if east_indies {
        dump("gamelog-run38-islands-start.txt")
            .map(crate::capture::read)
            .into_iter()
            .collect()
    } else {
        sibling_texts()
    };
    let sib_logs: Vec<Log> = sib_texts.iter().map(|t| Log::parse(t)).collect();
    let sib_inits: Vec<Initial> = sib_logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = sib_inits.iter().collect();
    borrow_from_siblings(&mut init, &refs);
    if east_indies {
        borrow_pasture(&mut init, &trace);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    let difficulty = built.sim.lobby.difficulty;
    let last = trace.frames.last().map_or(0, |(n, _)| *n);
    let window = site_window();
    for f in 0..last {
        built.tick();
        debug_leader(&built, f);
        debug_armies(&built, f);
        // `RON_DEBUG_UNIT`, on every frame of the long walk: a unit's
        // history before a widening's first block (item 1106's `1/14`).
        debug_watch(&built, f + 1);
        probe(f, &built);
        if window.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
            for (label, who) in attributed_sites(&built) {
                eprintln!("  f{f} {who}: {label}");
            }
            let theirs = trace.labels(f);
            if theirs != built.frame_sites.last().map_or(&[][..], |(_, v)| v) {
                eprintln!("  f{f} PARTS: theirs {theirs:?}");
            }
        }
    }
    let sequence = built
        .frame_sites
        .iter()
        .find(|(f, ours)| *ours != trace.labels(*f))
        .map_or(last, |(f, _)| *f);
    let count = built
        .frame_sites
        .iter()
        .find(|(f, ours)| ours.len() != trace.labels(*f).len())
        .map_or(last, |(f, _)| *f);
    eprintln!("{gamelog}: word parts at {count}, sequence at {sequence} of {last}");
    for (f, ours) in built.frame_sites.iter() {
        if *f != count && *f != sequence {
            continue;
        }
        let theirs = trace.labels(*f);
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i) != theirs.get(i))
            .unwrap_or(0);
        eprintln!(
            "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
            ours.len(),
            theirs.len(),
            ours.get(at),
            theirs.get(at)
        );
        if std::env::var("RON_DEBUG_SITES").is_ok() {
            for (i, l) in ours.iter().enumerate() {
                eprintln!("    ours  {i}: {l}");
            }
            // Each of the original's draws with the word before its step
            // (item 922: a seed makes every roll on the frame readable).
            for (i, (l, d)) in theirs.iter().zip(trace.frame_draws(*f)).enumerate() {
                eprintln!("    thrs  {i}: {l} (seed {:08x})", d.seed);
            }
        }
    }
    let endpoint = log.final_state().map(|fin| {
        let r = crate::diff::endpoint::walk_to_close(&mut built, &fin, 8);
        eprintln!(
            "{gamelog}: endpoint {}: {} compared, {} off, {} unlinked, {} extra, {} torn; \
             builds {}/{} unlinked/diverged, cities {}/{}",
            r.frame,
            r.compared,
            r.off.len(),
            r.unlinked.len(),
            r.extra.len(),
            r.torn.len(),
            r.build_unlinked,
            r.build_diverged,
            r.city_unlinked,
            r.city_diverged,
        );
        r
    });
    Some(Walk {
        count,
        sequence,
        last,
        difficulty,
        endpoint,
    })
}

/// **The compared pin's window** (items 1061 and 1106, DECISIONS 54 §2):
/// the newest pair's lower map's word — East Indies at Toughest,
/// [`SECOND_WORD_EAST_INDIES`] — its block and two on either side, on the
/// word's own widening walked from run346's start with the group record
/// and the attack order's row ([`widen_records`]). `coverage`'s compared
/// pin walks these blocks with the recorder on. Item 1061 moved it to
/// Great Lakes' word on run373, and it followed that word to run403's last
/// blocks; item 1099 closed Great Lakes at its end, 5930, and item 1106
/// moved the walk here (`docs/GROUPS.md` §33.3); item 1115 moved the word
/// to 5975, and the walk to run414; item 1120 moved the word to 6151,
/// inside it; item 1127 moved it to 6321, and the walk to run419; item 1143
/// moved it to 6609, and the walk to run420; item 1156 moved it to 6743,
/// inside it; item 1164 moved it to 7382, and the walk to run425; item
/// 1174 moved it to 7512, inside it; item 1185 moved it to 8519, and the
/// walk to run439; item 1191 moved it to 8820, and the walk to run445;
/// item 1197 moved it to 8907, inside it; item 1214 moved it to 10183,
/// and the walk to run462; item 1228 moved it to 10185, inside it; item
/// 1243 moved it to 10985, and the walk to run480; item 1264 moved it to
/// 11328, and the walk to run490; item 1281 moved it to 11549, inside it;
/// item 1297 moved it to 11637, and the walk to run506; item 1302 moved
/// it to 12582, and the walk to run508; item 1326 moved it to 13385, and
/// the walk to run523; item 1341 moved it to 14141, and the walk to
/// run535; item 1351 moved it to 15862, and the walk to run544; item 1362
/// moved it to 15883, inside it; item 1370 moved it to 15985, inside it;
/// item 1377 moved it to 16009, inside it.
/// `None`
/// when the captures are
/// not on this machine.
pub(crate) fn east_indies_word_window() -> Option<crate::diff::harness::tests::Widened> {
    let word = SECOND_WORD_EAST_INDIES;
    // Frame `f` writes block `f + 1`, and the walk reads `first..=tail + 1`.
    crate::diff::harness::tests::widen_east_indies_on(
        (
            "gamelog-run346-islands-toughest-24k-trace.txt",
            "rontrace-run346.log",
        ),
        "the second pair's word's window",
        "gamelog-run544-islands-toughest-15862.txt",
        (word - 1, word + 2),
        &[word + 1],
        true,
        true,
    )
}

/// **The group record and the attack order's row, both directions** (item
/// 1061, DECISIONS 54 §2; parked 1062): what the second pair's widening
/// did not compare while its word stood on a group's order. On one block,
/// for every player's 64 pool slots that either side holds, the whole
/// `GROUPDATA` record — the list, `num`, `army`, `form`, `order_num`,
/// `ox`/`oy`, `o_dist`, `o_angle`, `facing`, `form_num`, `speed`,
/// `new_speed`, `stamp`, `buildings`, `disband`, `priority`, `role` and each
/// slot's `off`, `curr` and `angle` — keyed `(who, -3, "group:<id>.<field>")`;
/// and on every unit, each order slot both sides hold as an `ATTACKORDER`,
/// its own row past the target — `mandatory`, `defensive`, `in_range`,
/// `ever_in_range`, `new_ord`, `def_x`, `def_y` — keyed
/// `(who, o, "attack[<slot>].<field>")`; and each figure's aim, `ox` and
/// `whom`, keyed `(who, o, "g.<field>[<k>]")`. A slot one side holds alone
/// is a `held` row. Answers the rows compared.
///
/// `disband` and `priority` are compared against 0: this crate writes
/// neither, and a player-1 pool slot the original sets either on is the
/// finding. `role` is compared on an army's group only, where this crate
/// keeps the word (`Army::role`, `docs/ARMY.md` §3.1); a pushed group
/// carries none here. `think_frame` this crate does not carry.
pub(crate) fn widen_records(
    built: &Built,
    frame: &Frame,
    block: crate::gamelog::Block<'_>,
    players: usize,
    n: i64,
    here: &mut std::collections::BTreeMap<(i64, i64, String), (i64, String)>,
) -> usize {
    let mut rows = 0usize;
    let pool = crate::gamelog::groups(block);
    for who in 0..players as i64 {
        let Ok(w) = sim::Player::try_from(who) else {
            continue;
        };
        for slot in 0..64u8 {
            let id = who * 64 + i64::from(slot);
            let theirs = pool.iter().find(|g| g.id == id).filter(|g| g.num > 0);
            let army = built
                .sim
                .armies
                .get(who as usize)
                .into_iter()
                .flat_map(|x| x.list.iter())
                .find(|a| a.valid && a.group.pool == Some(slot));
            let mut list = built.sim.pool_list(w, slot);
            let mut ours = built.sim.pool_state(w, slot);
            let mut buildings = false;
            if list.is_empty()
                && ours.is_none()
                && let Some((st, b)) = built.sim.pool_building_group(w, slot)
            {
                list = b;
                ours = Some(st);
                buildings = true;
            }
            if theirs.is_none() && list.is_empty() {
                continue;
            }
            let mut row = |key: &str, o: String, t: String| {
                rows += 1;
                if o != t {
                    here.entry((who, -3, format!("group:{id}.{key}")))
                        .or_insert((n, format!("ours {o} theirs {t}")));
                }
            };
            let (Some(t), Some(o)) = (theirs, ours) else {
                row(
                    "held",
                    format!("{list:?}"),
                    format!(
                        "{:?}",
                        theirs.map(|t| t.members.iter().map(|m| m.o).collect::<Vec<_>>())
                    ),
                );
                continue;
            };
            let tl: Vec<i64> = t.members.iter().map(|m| m.o).take(t.num as usize).collect();
            let ol: Vec<i64> = list.iter().map(|&x| i64::from(x)).collect();
            compared::note(
                "GroupDump",
                &[
                    "num",
                    "army",
                    "form",
                    "order_num",
                    "ox",
                    "oy",
                    "o_dist",
                    "o_angle",
                    "facing",
                    "form_num",
                    "speed",
                    "new_speed",
                    "stamp",
                    "buildings",
                    "disband",
                    "priority",
                ],
            );
            // The slot's identity and its list, member for member: what
            // the second pair's Great Lakes walk registered through its
            // pool-list pass, which East Indies' walk does not run.
            compared::note("GroupDump", &["id", "who", "members"]);
            compared::note("GroupMemberDump", &["o"]);
            row("list", format!("{ol:?}"), format!("{tl:?}"));
            for (k, ov, tv) in [
                ("num", ol.len() as i64, t.num),
                ("army", army.map_or(-1, |a| i64::from(a.army)), t.army),
                ("form", i64::from(o.form), t.form),
                ("order_num", i64::from(o.order_num), t.order_num),
                ("ox", i64::from(o.o.x), t.ox),
                ("oy", i64::from(o.o.y), t.oy),
                ("o_dist", i64::from(o.o_dist), t.o_dist),
                ("o_angle", i64::from(o.o_angle.0), t.o_angle),
                ("facing", i64::from(o.facing), t.facing),
                ("form_num", i64::from(o.form_num), t.form_num),
                ("speed", i64::from(o.speed), t.speed),
                ("new_speed", i64::from(o.new_speed), t.new_speed),
                ("stamp", o.stamp, t.stamp),
                ("buildings", i64::from(buildings), t.buildings),
                ("disband", 0, t.disband),
                ("priority", 0, t.priority),
            ] {
                row(k, ov.to_string(), tv.to_string());
            }
            if let Some(a) = army {
                compared::note("GroupDump", &["role"]);
                row("role", a.role.to_string(), t.role.to_string());
            }
            let slots = (o.form_num.max(0) as usize).max(t.form_num.max(0) as usize);
            if slots > 0 {
                compared::note(
                    "GroupMemberDump",
                    &["off_x", "off_y", "curr_x", "curr_y", "angle"],
                );
            }
            for i in 0..slots {
                let tm = t.members.get(i);
                row(
                    &format!("off[{i}]"),
                    format!("{:?}", o.off.get(i)),
                    format!("{:?}", tm.map(|m| (m.off_x, m.off_y))),
                );
                row(
                    &format!("curr[{i}]"),
                    format!("{:?}", o.curr.get(i).map(|p| (p.x, p.y))),
                    format!("{:?}", tm.map(|m| (m.curr_x, m.curr_y))),
                );
                row(
                    &format!("angle[{i}]"),
                    format!("{:?}", o.angles.get(i)),
                    format!("{:?}", tm.map(|m| m.angle)),
                );
            }
        }
    }
    for them in &frame.units {
        if !(0..players as i64).contains(&them.who) {
            continue;
        }
        let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
            continue;
        };
        let Some(u) = built.sim.unit_by_o(who, o) else {
            continue;
        };
        let un = &built.sim.units[u];
        // `RON_STACKS=1` prints every unit's order stack front first on
        // both sides, the kind with an attack's target, `in_range` and
        // `new_ord` — the instance list a chase's push or pop asks for.
        if std::env::var_os("RON_STACKS").is_some() {
            let ours: Vec<String> = un
                .orders
                .iter()
                .map(|x| match x.body {
                    sim::orders::Body::Attack(a) => format!(
                        "10{:?}r{}n{}",
                        un.combat.target.map(|t| match t {
                            sim::combat::Obj::Unit(t) => {
                                (
                                    i64::from(built.sim.units[t].owner),
                                    i64::from(built.sim.units[t].index),
                                )
                            }
                            sim::combat::Obj::Building(b) => (
                                i64::from(built.sim.buildings[b].owner),
                                i64::from(built.sim.buildings[b].index),
                            ),
                        }),
                        u8::from(a.in_range),
                        u8::from(a.new_ord)
                    ),
                    _ => x.index().to_string(),
                })
                .collect();
            let theirs: Vec<String> = them
                .orders_front_first()
                .map(|x| {
                    if x.index == 10 {
                        format!(
                            "10{:?}r{}n{}",
                            x.whom.zip(x.ox),
                            x.in_range.unwrap_or(-1),
                            x.new_ord.unwrap_or(-1)
                        )
                    } else {
                        x.index.to_string()
                    }
                })
                .collect();
            if ours
                .iter()
                .chain(theirs.iter())
                .any(|k| k.starts_with("10"))
            {
                eprintln!(
                    "  stacks {n} {who}/{o} ours {} ({},{}) | theirs {} ({},{})",
                    ours.join(","),
                    un.pos.x,
                    un.pos.y,
                    theirs.join(","),
                    them.pos.x,
                    them.pos.y
                );
            }
        }
        // **What each figure is aimed at** (`GuyData +0x8e`/`+0x9f`,
        // printed `ox`/`whom`): chapter one's widening read it on a
        // player's fight, and a war window is where it is written.
        for (k, g) in them.guys.iter().enumerate() {
            let Some(og) = un.guys.get(k) else { continue };
            let aim = match og.aim {
                None => (-1, -1),
                Some(sim::combat::Obj::Unit(t)) => {
                    let tu = &built.sim.units[t];
                    (i64::from(tu.owner), i64::from(tu.index))
                }
                // A building by its `(owner, index)`, the identity
                // `widen_block` links on: `build_ids` names only the ones
                // the start dump placed.
                Some(sim::combat::Obj::Building(b)) => {
                    let x = &built.sim.buildings[b];
                    (i64::from(x.owner), i64::from(x.index))
                }
            };
            for (name, mine, dumped) in [("whom", aim.0, g.whom), ("ox", aim.1, g.ox)] {
                let Some(dumped) = dumped else { continue };
                compared::note("Guy", &[name]);
                rows += 1;
                if mine != dumped {
                    here.entry((them.who, them.o, format!("g.{name}[{k}]")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
        let mut first = true;
        for (slot, (ours, od)) in un.orders.iter().zip(them.orders_front_first()).enumerate() {
            let sim::orders::Body::Attack(a) = ours.body else {
                continue;
            };
            if od.index != i64::from(sim::orders::index::ATTACK) {
                continue;
            }
            let def = a.def.unwrap_or(sim::Pos::new(-1, -1));
            let mut fields = vec![
                ("defensive", i64::from(a.defensive), od.defensive),
                ("in_range", i64::from(a.in_range), od.in_range),
                (
                    "ever_in_range",
                    i64::from(a.ever_in_range),
                    od.ever_in_range,
                ),
                ("new_ord", i64::from(a.new_ord), od.new_ord),
                ("def_x", i64::from(def.x), od.def_x),
                ("def_y", i64::from(def.y), od.def_y),
            ];
            // `mandatory` lives on the unit (`combat::State`), so it is
            // the front-most attack's.
            if first {
                fields.push(("mandatory", i64::from(un.combat.mandatory), od.mandatory));
                first = false;
            }
            for (name, mine, dumped) in fields {
                let Some(dumped) = dumped else { continue };
                compared::note("OrderDump", &[name]);
                rows += 1;
                if mine != dumped {
                    here.entry((them.who, them.o, format!("attack[{slot}].{name}")))
                        .or_insert((n, format!("ours {mine} theirs {dumped}")));
                }
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::harness::tests::widen_east_indies_on;

    /// **The second pair's East Indies word, widened whole** (item 971):
    /// run349 is run346's game at run299's per-frame detail over blocks
    /// 1..250 (`rngcmp.py`: 263 frames, 0 differing), walked from run346's
    /// start by [`widen_east_indies_on`] — every record, every unit, both
    /// leaders, both directions — with the word's block (frame 0 writes
    /// block 1) and the two after it kept standing. It keeps the move's
    /// value diff; East Indies' next word, 10, is inside its window.
    #[test]
    fn run349_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run349",
            "gamelog-run349-islands-toughest-open.txt",
            WIDENING_SECOND_EAST_INDIES,
            &[2],
            true,
            false,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 250, "run349 whole: blocks 1..250");
        pin!(
            w.missing.is_empty(),
            "run349 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **item 971's value diff**, `docs/AI.md` §80.5: before
        // `think_spellcaster`'s coin the AI scout `1/0`'s walk target on
        // block 1 was 41976/36600 here against 35832/42744, and 28 of its
        // rows parted on blocks 1..3. **Item 979's** (§81): with run38's
        // frame words — an Easiest game's — no longer installed over this
        // one, the scout's walk parting (`orders_x` 38136 against 41976 on
        // block 97) closed too; its first row was its sight count on block
        // 202, ours 6 against 4 — the Science epoch reaching the derivation
        // a frame before the original refreshed its cache — and **item
        // 1354** closed that with the cache (`docs/VISION.md` §2).
        pin_eq!(row(1, 0, "orders_x"), None, "the scout's walk agrees");
        pin_eq!(row(1, 0, "mylos"), None, "the scout's sight count agrees");
        pin!(
            !w.firsts.keys().any(|(who, o, _)| (*who, *o) == (1, 0)),
            "none of the scout's rows parts"
        );
        // **And run38's clocks**: block 1's 46 gaia `cur_anim` rows and
        // `1/1`'s `g.end_time[0]` 232 against 33 were the sibling's
        // figures, installed at frame 0's end; both closed with the words.
        pin_eq!(row(1, 1, "g.end_time[0]"), None, "an AI citizen's clock");
        pin!(
            !w.firsts
                .iter()
                .any(|((who, _, what), _)| *who == 8 && what == "gaia:cur_anim"),
            "no gaia animal's clock parts"
        );
        // **What stands**: block 1's 67 keys (player 0's census,
        // `SITE.reg`, `form`, the city record — `docs/AI.md` §33.4's
        // families) and block 2's tech stamps, `tech_frame` and two
        // `tech_cat_frame`s, which the original's `Build::queue_up` writes
        // and this crate does not (§33.4's named unmodelled state).
        pin_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("2: ours 0 theirs 1"),
            "the original's tech stamp, standing"
        );
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 66), (2, 3)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, widened whole** (item 971):
    /// run350 is run347's game at run349's detail over blocks 1..250, walked
    /// from run347's start by [`widen_great_lakes_on`]. The word is frame 1,
    /// which writes block 2.
    #[test]
    fn run350_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run350",
            &[("gamelog-run350-greatlakes-toughest-open.txt", 1)],
            WIDENING_SECOND_GREAT_LAKES,
            1,
            &[2],
            false,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 250, "run350 whole: blocks 1..250");
        pin!(
            w.missing.is_empty(),
            "run350 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 1 stands** with frame 0's draws agreeing: 55 keys, the
        // first pair's families (player 0's census, `SITE.reg`, `form`, the
        // city record; `docs/AI.md` §33.4) and nothing of who=1's economy.
        // **Item 979's value diff** (§81): before the lobby gate, run12's
        // frame-0 word — the Easiest game's, one draw short — was installed
        // over this game's, frame 1's rush roll read 9 where the trace's
        // seeds give 3, and who=1's block 2 parted on 130 keys: `script_step`
        // 11 here against 6, timber 28 against 92, wealth 50 against 100,
        // `gatherers` 4 against 3, `1/2001`'s 90 gather slots, citizen
        // `1/2`'s walk and the rush's second farm `2007`. Every one of them
        // agrees now.
        for what in [
            "leader:script_step",
            "leader:bucket[1:timber]",
            "leader:bucket[2:wealth]",
            "leader:gatherers",
        ] {
            pin_eq!(row(1, -1, what), None, "who=1's {what} agrees");
        }
        pin!(
            !w.firsts.iter().any(|((who, o, _), (f, _))| *who == 1
                && *f >= 2
                && [2, 2001, 2005, 2006, 2007].contains(o)),
            "the citizen, the camp's slots and the rush's sites agree past block 1"
        );
        // **What stands on the old word's block**: the original's tech
        // stamps (`Build::queue_up` writes `tech_frame` and
        // `tech_cat_frame[cat]`; §33.4's named unmodelled state) and two
        // pools of the script's buildings, the library `2005` in 64 and the
        // city `2000` in 66, which this crate never seats. No draw reads
        // either before the word.
        pin_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("2: ours 0 theirs 1"),
            "the Art of War's stamp"
        );
        pin_eq!(
            row(1, -2, "pool:64").as_deref(),
            Some("2: ours [] theirs [2005]"),
            "the original's pool 64"
        );
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 54), (2, 4), (8, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 1576, widened whole** (item
    /// 979): run352 is run346's game at run349's detail over blocks
    /// 1571..1827 (`rngcmp.py`: 1841 frames, 0 differing), walked from
    /// run346's own start. The word's frame writes block 1577.
    #[test]
    fn run352_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run352",
            "gamelog-run352-islands-toughest-1576.txt",
            WIDENING_SECOND_EAST_INDIES_1576,
            &[1_577],
            true,
            false,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 257, "run352 whole: blocks 1571..1827");
        pin!(
            w.missing.is_empty(),
            "run352 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word's block, 1577, as item 979 found it** (`docs/AI.md`
        // §81.4): the building `1/2009` parted on 91 of its 93 keys — its
        // gather tiles in another order, damage 1 here against 0, city −1
        // against 1 — and the citizen `1/2` walked to another cell
        // (`orders_x` 31944 against 37752). **Item 989's value diff**
        // (§82): the script's `place_woodcutter` places a camp at (38016,
        // 36480) on frame 976 and destroys it the same frame, and
        // `Build::close` gives its 48 tiles back; this crate kept them
        // marked, so the camp re-placed on 1576 went to (31680, 34752) with
        // 62 tiles (248 draws against 192). And the harness linked the
        // original's live `2009` to this crate's dead frame-976 site, which
        // is where damage 1 and city −1 came from. Every row of the word's
        // block now agrees, and so does capital `2000`'s `ter[1]` on 1571,
        // 0 against 2 → 2 on both: the census's `gather_at` had read the
        // dead camp's marks.
        pin!(
            !w.firsts.iter().any(|((who, o, _), (f, _))| *who == 1
                && [2, 2009].contains(o)
                && (1_572..=1_577).contains(f)),
            "the camp and the citizen agree from 1572 through the word's block"
        );
        pin_eq!(row(1, 2000, "city:ter[1]"), None, "the capital's timber");
        // **What stands, drawing nothing before the next word** (5606):
        // city `2007`'s site picture, one cell apart from its sweep of 1575
        // — `filled` one high here and the three `space` counts one low, so
        // `reg_land[11]` 102 against 103 — and, from the sweep before
        // (block 1571), the same city counted by the original while still a
        // site (`land` 9, `reg_cities[11]` 2) and not here. No mechanism is
        // named (DECISIONS 42).
        pin_eq!(
            row(1, -1, "leader:reg_land[11]").as_deref(),
            Some("1576: ours 102 theirs 103"),
            "the block before the word"
        );
        pin_eq!(
            row(1, 2007, "city:space[0]").as_deref(),
            Some("1576: ours 64 theirs 65"),
            "the city's site picture"
        );
        // Item 1106 closed two on 1571 (82 → 80): who=1's
        // `reg_cities[11]`, 1 against 2 — `City::init` counts a city on the
        // frame it is made, which the note above had read as a site the
        // original counted — and who=0's `gather_stamp`, 1424 against 1432
        // (`docs/AI.md` §84).
        pin_eq!(
            row(1, -1, "leader:reg_cities[11]"),
            None,
            "the new city counts at once"
        );
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            // Item 1377: 1571 74 → 73, who=1's `defense` 0 against 1.
            [(1571, 71), (1576, 5), (1601, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, 3776, widened whole** (item
    /// 979): run355 is run347's game at run350's detail over blocks
    /// 3771..4027 (`rngcmp.py`: 4041 frames, 0 differing), walked from
    /// run347's start. The word's frame writes block 3777.
    #[test]
    fn run355_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run355",
            &[(
                "gamelog-run355-greatlakes-toughest-3776.txt",
                WIDENING_SECOND_GREAT_LAKES_3776.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_3776,
            1,
            &[3_777],
            false,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 257, "run355 whole: blocks 3771..4027");
        pin!(
            w.missing.is_empty(),
            "run355 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word's block, 3777, as item 979 found it** (`docs/AI.md`
        // §81.4), East Indies' shape: a building who=1 places on the word's
        // frame, `1/2010` — 79 of its 80 keys its gather slots' tiles, and
        // damage 1 here against 0 — with `2007`'s `city_down` pointing at
        // it here and not there, the city `2000` one gatherer short (7
        // against 8), and citizens `1/6` and `1/28` on other orders.
        // **Item 989's value diff** (§82): the same close tail — a camp the
        // script destroyed kept its ground marked here — and the same dead
        // site linked in place of the live `2010`. All 127 keys of the
        // word's block agree now, and nothing parts on 3772..3804.
        pin!(
            !w.firsts
                .iter()
                .any(|((who, _, _), (f, _))| *who == 1 && (3772..3805).contains(f)),
            "who=1 agrees from the window's second block through 3804"
        );
        pin_eq!(row(1, 2000, "city:gatherers"), None, "the city's census");
        pin_eq!(row(1, 6, "orders_x"), None, "a citizen's order");
        // **What parts next, drawing nothing before the next word**
        // (4555): a city site `1/2009` the original counts in its sweep of
        // 3804 (`land` 9, `filled` 1) and this crate does not, and who=1's
        // `territory` 488 here against 290 — East Indies' `2007` on its
        // block 1571 has the same shape. No mechanism is named (DECISIONS
        // 42).
        pin_eq!(
            row(1, 2009, "city:land").as_deref(),
            Some("3805: ours 0 theirs 9"),
            "the city site's sweep"
        );
        // **Closed by item 1106**: who=1's `territory` is summed when the
        // border pass ends (`docs/AI.md` §84), and agrees on 3805.
        pin_eq!(
            row(1, -1, "leader:territory"),
            None,
            "and who=1's territory"
        );
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            // Item 1377: 3805 3 → 2, who=1's `defense` 0 against 1.
            [(3771, 71), (3801, 1), (3805, 2)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 5606, widened whole** (item
    /// 989): run357 is run346's game at run352's detail over blocks
    /// 5601..5857, walked from run346's own start. The word's frame writes
    /// block 5607.
    #[test]
    fn run357_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run357",
            "gamelog-run357-islands-toughest-5606.txt",
            WIDENING_SECOND_EAST_INDIES_5606,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // `RON_FIRSTS=1` prints every key's first parting on the window.
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run357 whole: blocks 5601..5857");
        pin!(
            w.missing.is_empty(),
            "run357 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Item 989's word, 5606: its block before, 5605, parted on one
        // key** (`docs/AI.md` §82.4, §84): `1/14`'s move `dest` 0 here
        // against 1, its path's third slot on 5607, and on 5601 its
        // position and route. **Since item 1106 none parts**: the citizen
        // that finished the city `1/2017` on tick 5517 takes its colonist
        // move on 5521, when the border pass reaches its cell, and not on
        // 5518 (run407's packet).
        for what in ["order:move.dest", "path[2].to", "pos", "order:move.y"] {
            pin_eq!(row(1, 14, what), None, "`1/14`'s {what}");
        }
        // **Since item 1106 the walk compares the group record and the
        // attack order's row** (`widen_records`). Every pool slot reads
        // `form` 0 here against 9, the family the units' `form` rows are,
        // and slots 69 and 71's point −1 against 0, a building group's
        // (`docs/GROUPS.md` §33.2). **Slot 67 is `1/14`'s alone**: before
        // the border pass it stood on 5601 with `ox` 29568 against 30336
        // and `stamp` 5518 against 5521, and now agrees but for `form`.
        pin_eq!(row(1, -3, "group:67.ox"), None, "`1/14`'s group's point");
        pin_eq!(row(1, -3, "group:67.stamp"), None, "and its frame");
        // Item 1330: and `form` agrees, a citizen born in form 9
        // (`Unit::init`, `docs/GROUPS.md` §24.3).
        pin_eq!(
            row(1, -3, "group:67.form").as_deref(),
            None,
            "the family's row stands"
        );
        // **The word 5773's block, 5774** (item 1106): the Caravan `1/33`,
        // trained on 5772, idled here (`idle` 99, no order) where the
        // original held a route of two orders to (39288, 40056). **Since
        // item 1115 none of its rows part** (`docs/CARAVAN.md` §11):
        // `do_trade` takes Newcastle across the water when the caravan
        // `can_transport`, the trade order carries no bit 4, and the move
        // to London faces its bearing, 546111488.
        for what in [
            "orders.len",
            "order:flags",
            "order:action",
            "order:move.angle",
            "dest_angle",
        ] {
            pin_eq!(row(1, 33, what), None, "`1/33`'s {what}");
        }
        pin_eq!(
            row(1, 2017, "city:vans.length"),
            None,
            "and the new city's caravan list"
        );
        // **Standing: the census's newborn lag** (item 1115, parked): the
        // original counts a caravan in `caras` at `Unit::set_type`
        // (`0x980` +1), this crate at its next census sweep, so who=1 reads
        // 1 against 2 on blocks 5772..5774 and agrees from 5775.
        pin_eq!(
            row(1, -1, "leader:caras").as_deref(),
            None,
            "the census lags the birth"
        );
        // **And the first caravan's** (item 1115): `1/15`, London to Norwich,
        // walked its legs facing `find_angle(1, 1)` here and its bearing
        // there, and carried the trade order's bit 4; block 5601's four
        // rows (`order:move.angle`, `dest_angle`, `order:flags`,
        // `order:action`) closed with `1/33`'s, 139 → 135. Item 1174: 135 →
        // 130, who=1's `escrow` on its five goods, which this crate never
        // fed (`docs/AI.md` §98).
        for what in ["order:move.angle", "dest_angle", "order:flags"] {
            pin_eq!(row(1, 15, what), None, "`1/15`'s {what}");
        }
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(5601, 130), (5682, 5), (5704, 2)].
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(5601, 93), (5682, 3), (5713, 2)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 5975, widened whole** (item
    /// 1115): run414 is run346's game at run357's detail over blocks
    /// 5970..6226, walked from run346's own start. The word's frame writes
    /// block 5976. **Since item 1120 the word is 6151**, block 6152, inside
    /// the same window (182 blocks after its first and 74 before its last),
    /// so this capture is its widening too. **Since item 1127 the word is
    /// 6321**, block 6322, past this window: run419 is its widening.
    #[test]
    fn run414_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run414",
            "gamelog-run414-islands-toughest-5975.txt",
            WIDENING_SECOND_EAST_INDIES_5975,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 257, "run414 whole: blocks 5970..6226");
        pin!(
            w.missing.is_empty(),
            "run414 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 5975** (item 1115) **closed on item 1120**
        // (`docs/VISION.md` §11): the barge `1/36`, born on 5878, lights its
        // whole disc on the water, run415's two half-cells (90, 80) and
        // (91, 81) with it, and the AI sea scout `1/35`'s region scan
        // accepts 45 cells, as the original's does. The newborn `1/40`'s
        // figure clocks, which read one seed late on 5976, agree.
        pin_eq!(row(1, 35, "pos"), None, "the scout itself agrees");
        for g in 0..3 {
            pin_eq!(
                row(1, 40, &format!("g.cur_time[{g}]")),
                None,
                "the newborn's figure {g}"
            );
        }
        // **The word 6151** (item 1120) **closed on item 1127**
        // (`docs/COLLISION.md` §19): on its block 6152 the AI Caravan
        // `1/15` stood against the Caravan `1/33` here (`pos` (35597,37251)
        // against (35617,37267), `collide_o` 33 against -1, `half_step` 0
        // against 1) where the original walks it through: both actions are
        // `TRADE_ROUTE` and both fronts a move, `detect_unit_collision`'s
        // first soft row. All 35 of its rows agree.
        for what in ["pos", "collide_o", "half_step", "g.stopped[0]"] {
            pin_eq!(row(1, 15, what), None, "`1/15`'s {what}");
        }
        // Standing in the whole window, which ends before the word 6321
        // (item 1127): the census's newborn lag (parked 1122) and `1/40`'s
        // `form` (5976), who=1's `MAKE[7].city` (5982), `gather_stamp`
        // (5984), who=0's `production_step` (6001), the passengers `1/23`'s
        // and `1/22`'s figure `avg_speed` and `mirror` (6026, 6069), group
        // 74's `form` (6071), `1/31`'s `path_recursion` (6074), the newborn
        // `1/41`'s `form`, figure angle and `orders_x/y` (6111), group 72's
        // speed (6135), who=1's `MAKE[4].val` 198000 against 202000
        // (6182), and the newborns `1/42`'s and `1/43`'s `form` -1 against
        // 0 (6191, 6218).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SECOND_WORD_EAST_INDIES + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [
                (5970, 103),
                (6001, 1),
                (6074, 1),
                (6111, 3),
                (6135, 2),
                (6182, 1),
                (6191, 1),
                (6218, 1)
            ],
            "the blocks keys first part on, the whole window"
        );
        // Item 1164 took the passengers' figure `avg_speed` (6026, 6069;
        // 176 → 174): put ashore at the average they boarded with
        // (`docs/TRANSPORT.md` §6.4); their `mirror` stands. Item 1174:
        // 174 → 169 and the first block 156 → 151, who=1's `escrow` on its
        // five goods (`docs/AI.md` §98).
        // Item 1291 took 2: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 167.
        pin_eq!(w.firsts.len(), 113, "every key parted on run414");
    }

    /// **The second pair's East Indies word, 6321, widened whole** (item
    /// 1127): run419 is run346's game at run414's detail over blocks
    /// 6316..6572, walked from run346's own start. The word's frame writes
    /// block 6322. **Since item 1143 the word is 6609**, block 6610, past
    /// this window: run420 is its widening.
    #[test]
    fn run419_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run419",
            "gamelog-run419-islands-toughest-6321.txt",
            WIDENING_SECOND_EAST_INDIES_6321,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 257, "run419 whole: blocks 6316..6572");
        pin!(
            w.missing.is_empty(),
            "run419 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 6321** (item 1127) **closed on item 1143**
        // (`docs/ORDERS.md` §4.4): the barge `1/42` (born on 6190 by
        // run414), carrying the citizen `1/32`, took its final leg ashore on
        // 6271, in the gap 6227..6315 no dump covers, and the waypoint
        // take's region check flags it `| 4` there, so `find_path` keeps the
        // land goal. It sailed apart before — `pos` (38028,11677) against
        // (38016,11689) on 6316, its leg (38028,11542) against (38016,
        // 11136) — stood with no order on 6321, and still carried `1/32` on
        // 6322 where the original had landed it at (38040,11400). All of it
        // agrees; the barge's `form` -1 against 0 stands from run414's 6191.
        // The passenger's figure `avg_speed`, 0 against 11 on 6322, was what
        // was left of the landing until item 1164 put it ashore at the
        // average it boarded with (`docs/TRANSPORT.md` §6.4): it agrees.
        for what in ["pos", "orders.len", "extra"] {
            pin_eq!(row(1, 42, what), None, "`1/42`'s {what}");
        }
        for what in ["inside", "pos"] {
            pin_eq!(row(1, 32, what), None, "`1/32`'s {what}");
        }
        pin_eq!(
            row(1, 32, "g.avg_speed[0]"),
            None,
            "the passenger's figure speed, put ashore"
        );
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter().take(6).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            // Item 1330: the first block 149 → 106, `6334`'s one row gone
            // and `6417`'s 3 → 2, the births' `form` (`docs/GROUPS.md` §24.3).
            [
                (6316, 104),
                (6393, 2),
                (6401, 1),
                (6417, 2),
                (6434, 2),
                (6449, 2)
            ],
            "the blocks keys first part on, the first six"
        );
        // Item 1164: 317 → 304, the landing's rows and what followed them.
        // Item 1174: 304 → 299 and the first block 160 → 155, who=1's
        // `escrow` on its five goods (`docs/AI.md` §98).
        // Item 1197: 299 → 182, the dead transports' numbers held
        // (`docs/COMBAT.md` §59.3). Item 1228: 182 → 179, who=1's
        // Merchant Fleets counted (`docs/TRANSPORT.md` §16).
        // Item 1291 took 4: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 175.
        pin_eq!(w.firsts.len(), 119, "every key parted on run419");
    }

    /// **The second pair's East Indies word, 6609, widened whole** (item
    /// 1143): run420 is run346's game at run419's detail over blocks
    /// 6604..6860, walked from run346's own start. The word's frame writes
    /// block 6610. **Since item 1156 the word is 6743**, block 6744, inside
    /// the same window (140 blocks after its first and 116 before its last).
    /// **Since item 1164 it is 7382**, past the window's last block 6860,
    /// widened on run425.
    #[test]
    fn run420_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run420",
            "gamelog-run420-islands-toughest-6609.txt",
            WIDENING_SECOND_EAST_INDIES_6609,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 257, "run420 whole: blocks 6604..6860");
        pin!(
            w.missing.is_empty(),
            "run420 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The first block stands on 249 keys**: 31 frames past run419's
        // last block, the gap 6573..6603 no dump covers.
        //
        // **The word 6609** (item 1143) **closed on item 1156**
        // (`docs/SCOUT.md` §8.1): the AI citizen `1/28` arrived at
        // (29568,24192) on 6576 and its region scan took cell (37,32) at 38,
        // because `find_unit_ordered` asked about `1/22` by its body; the
        // original measures a sibling's `orders_x`/`orders_y` (`0065be35`),
        // (27384,25080), which rejects (37,32), and takes (34,29) at 128. Its `pos` stood apart on 6604 — (29017,24361)
        // against (28893,24233) — its move's `angle` with it, and on 6610
        // the original stood it against the animal `8/3` (`collide_o` 3)
        // where ours walked on. All of it agrees.
        for what in ["pos", "order:move.angle", "collide_o", "path:length"] {
            pin_eq!(row(1, 28, what), None, "`1/28`'s {what}");
        }
        pin_eq!(
            row(1, -3, "group:65.o_angle"),
            None,
            "its one-member group's angle"
        );
        pin_eq!(
            row(8, 3, "gaia:pos"),
            None,
            "the animal agrees through the window since item 1164"
        );
        // **The word 6743** (item 1156) **closed on item 1164**
        // (`docs/TRANSPORT.md` §6.4, `docs/AI.md` §95): the AI merchant
        // `1/33`, carried in the barge that is `1/36` here and `1/38` there,
        // comes ashore on 6734. The original kept its guys' `avg_speed` 12
        // through the ride, ours zeroed it (6735: 0 against 12), and the
        // turn rate divides by it: the original turns to its path over nine
        // frames and faces it on 6745, ours on 6743, and walked on (`pos`
        // (38944,24864) against (38952,24840) on 6743). Its `orders_x/y`,
        // (37439,33407) against (38952,24840) on 6735, is `come_out`'s tail
        // `update_action` for a computer player's unit. All of it agrees.
        for what in [
            "orders_x",
            "orders_y",
            "pos",
            "g.avg_speed[0]",
            "g.angle[0]",
            "path:length",
        ] {
            pin_eq!(row(1, 33, what), None, "`1/33`'s {what}");
        }
        // What stands: the first block's 244 keys (249 until item 1174 fed
        // who=1's `escrow`, `docs/AI.md` §98), the barge identities
        // `1/36`/`1/38` (6718; 6735's unlinked/extra, the one that landed
        // `1/33` on each side), one-member groups' `speed`, and from 6781
        // player 1's make list (`MAKE[].city`), `gather_stamp` (6792) and
        // player 0's `production_step` (6801), ahead of the word 7382's
        // `Leader::use_market` against `Leader::make_stuff`.
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [
                (6604, 103),
                (6670, 2),
                (6671, 2),
                (6685, 4),
                (6718, 8),
                (6742, 2),
                (6769, 2),
                (6778, 2),
                (6789, 2),
                (6801, 1)
            ],
            "the blocks keys first part on, the whole window"
        );
        // Item 1197: 298 → 189, the dead transports' numbers held
        // (`docs/COMBAT.md` §59.3). Item 1228: 189 → 185, and the first
        // block's 156 → 152, who=1's Merchant Fleets counted (`docs/TRANSPORT.md` §16).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 184.
        pin_eq!(w.firsts.len(), 128, "every key parted on run420");
    }

    /// **The second pair's East Indies word, 7382, widened whole** (item
    /// 1164): run425 is run346's game at run420's detail over blocks
    /// 7377..7633, walked from run346's own start. The word's frame writes
    /// block 7383. **Since item 1174 the word is 7512**, block 7513, inside
    /// the same window (136 blocks after its first and 120 before its last).
    #[test]
    fn run425_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run425",
            "gamelog-run425-islands-toughest-7382.txt",
            WIDENING_SECOND_EAST_INDIES_7382,
            &[7_513],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run425 whole: blocks 7377..7633");
        pin!(
            w.missing.is_empty(),
            "run425 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 7382** (item 1164) **closed on item 1174** (`docs/AI.md`
        // §98): ours spent two `use_market` and three
        // `produce_building+0x1805` draws the original did not, on player
        // 1's make list priced at three sevenths — its `MAKE[].val` on 7379
        // ours 1800000 against 4194000 in slot 0. The research base is
        // `pop × 200 / cities`, and `pop` stood 3 against 7 on the first
        // block: London and Norwich level up to Large Cities with the
        // Medieval Age in the original (`gain_tech`'s TOWN arm) and stood
        // Small here. The buckets parted on 7383 (wealth 118 against 8)
        // because `escrow` stood 0 against 39/44/15/47/36: `do_gather`
        // fed it and nothing here did. All of it agrees now.
        pin_eq!(
            row(1, -1, "leader:MAKE[0].val"),
            None,
            "the make list's value"
        );
        pin_eq!(row(1, -1, "leader:pop"), None, "the cities' pop value");
        for g in ["0:food", "1:timber", "2:wealth", "3:knowledge", "4:metal"] {
            pin_eq!(
                row(1, -1, &format!("leader:escrow[{g}]")),
                None,
                "escrow {g}"
            );
            pin_eq!(
                row(1, -1, &format!("leader:bucket[{g}]")),
                None,
                "bucket {g}"
            );
        }
        // What stands to the new word: the first block's 217 keys, the city
        // shift (§52.2) on 7381, 7382 and 7384, and on 7384 the Units
        // step's slot 4 (a Merchant here against a Farm there) and
        // `known_rares` 2 against 0.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was Some("7381: ours 1 theirs 0").
        pin_eq!(
            row(1, -1, "leader:MAKE[1].city").as_deref(),
            None,
            "the city shift"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[4].t").as_deref(),
            None,
            "the Units step's slot 4"
        );
        // **The word 7512** (item 1174) **closed on item 1185**
        // (`docs/ROADS.md` §9.5): `World::set_blocked_at@006b4900` takes the
        // road off a tile it blocks, and this crate's did not. The Temple
        // `1/2025`, placed over the caravan road on 7385, started on 7479,
        // and its blocked tile (200, 202) kept its road here. On 7512 the
        // caravan `1/15` takes that tile's waypoint (38560, 38816): the
        // original finds plain ground under a building, verifies the route
        // and plans it again (3206 `calc_road_cost` draws), and strips
        // `0x20` from every waypoint — on block 7513 its 23 `path[].flags`
        // ours 33/32… against 1/0… → agreeing.
        pin_eq!(
            row(1, -1, "leader:caras").as_deref(),
            Some("7478: ours 3 theirs 2"),
            "a caravan more here"
        );
        pin_eq!(
            row(1, 15, "path[0].flags"),
            None,
            "the word's block: the caravan's path"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 7_513)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [
                (7377, 100),
                (7401, 1),
                (7419, 3),
                (7424, 2),
                (7478, 1),
                (7485, 3)
            ],
            "the blocks keys first part on, to the old word's"
        );
        // Item 1174: 1182 → 614. Item 1185: 614 → 317.
        // Item 1197: 317 → 187, the dead transports' numbers held
        // (`docs/COMBAT.md` §59.3). Item 1228: 187 → 183, the first block's
        // 161 → 158 and 7478's 2 → 1, who=1's Merchant Fleets counted (`docs/TRANSPORT.md` §16).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 174.
        pin_eq!(w.firsts.len(), 111, "every key parted on run425");
    }

    /// **The second pair's East Indies word 8519, widened whole** (item
    /// 1185): run439 is run346's game at run425's detail over blocks
    /// 8514..8770, walked from run346's own start — every dumped record on
    /// the word's block 8520 and the six before it, both directions.
    #[test]
    fn run439_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run439",
            "gamelog-run439-islands-toughest-8519.txt",
            WIDENING_SECOND_EAST_INDIES_8519,
            &[8_520],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run439 whole: blocks 8514..8770");
        pin!(
            w.missing.is_empty(),
            "run439 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 8519** (item 1185; no mechanism was named): ours 2
        // draws against 3, at index 0, ours `Guy::set_anim+0x97a <
        // Guy::inc_time+0x271`, the original `Guy::set_anim+0x97a <
        // Guy::move+0x19f`. The first block stands on 481 keys from the gap
        // 7634..8513, among them `1/68` and `1/69` each on the other's walk;
        // their moves' `dest` part on 8516 and 8519. **Item 1191 closed it**:
        // the original's extra draw is the boat `1/56`'s arrival stand on
        // the frame it steps ashore and dies (`docs/TRANSPORT.md` §6.4), and
        // ours gave the roll to `1/21`'s wrap — on the word's block 8520
        // `1/21`'s `g.cur_anim[0]` ours 26 against 25 and `g.end_time[0]`
        // 100 against 30 → agreeing.
        // **Item 1197 closed those two**: they were `1/68` and `1/69` on
        // each other's numbers — the three Longbowmen stood at `1/59`,
        // `1/68`, `1/69` here while the dead transports' numbers were not
        // held (`docs/COMBAT.md` §59.3).
        pin_eq!(
            row(1, 69, "order:move.dest").as_deref(),
            None,
            "`1/69`'s move agrees (item 1197)"
        );
        pin_eq!(
            row(1, 68, "order:move.dest").as_deref(),
            None,
            "and `1/68`'s"
        );
        pin_eq!(
            row(1, 21, "g.cur_anim[0]").as_deref(),
            None,
            "the word's block: `1/21`'s figure clock agrees (item 1191)"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(8514, 167)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 8_520)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(8514, 107)],
            "the blocks keys first part on, to the old word's block 8520"
        );
        // Item 1191: 1243 → 608. Item 1197: 608 → 198. Item 1228: 198 →
        // 194, and block 8514's 180 → 176, who=1's Merchant Fleets counted (`docs/TRANSPORT.md` §16).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 185.
        pin_eq!(w.firsts.len(), 120, "every key parted on run439");
    }

    /// **The second pair's East Indies word 8820, widened whole** (item
    /// 1191): run445 is run346's game at run439's detail over blocks
    /// 8815..9071, walked from run346's own start — every dumped record on
    /// the word's block 8821 and the six before it, both directions. **Since
    /// item 1197 the word is 8907**, block 8908, inside the same window (93
    /// blocks after its first and 163 before its last). **Since item 1214
    /// the word is 10183**, past the window; its widening is run462's.
    #[test]
    fn run445_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run445",
            "gamelog-run445-islands-toughest-8820.txt",
            WIDENING_SECOND_EAST_INDIES_8820,
            &[8908],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run445 whole: blocks 8815..9071");
        pin!(
            w.missing.is_empty(),
            "run445 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 8820** (item 1191; no mechanism was named): ours 2
        // draws against 3, at index 2, the original's third a
        // `Guy::set_anim+0x97a < Guy::inc_time+0x271` ours did not make. The
        // first block stood on 504 keys from the gap 8771..8814, among them
        // the figure clocks of two units the original wraps on 8820 —
        // `1/68`'s `g.cur_time[0]` ours 21 against 25 and `1/70`'s
        // `g.cur_anim[0]` ours 35 against 1. **Item 1197 closed it**
        // (`docs/COMBAT.md` §59.3): the Longbowmen `1/68`–`1/70` stood at
        // `1/59`, `1/68`, `1/69` here, because the transports that died
        // putting their passengers ashore in the gap did not hold their
        // numbers. Both clocks agreed past the word and parted on 8945;
        // **since item 1214 they agree through the window's end**, 9071.
        pin_eq!(
            row(1, 68, "g.cur_time[0]").as_deref(),
            None,
            "`1/68`'s clock agrees through the window (item 1214)"
        );
        pin_eq!(
            row(1, 70, "g.cur_anim[0]").as_deref(),
            None,
            "and `1/70`'s animation"
        );
        // **The word 8907** (item 1197; no mechanism is named): ours 2 draws
        // against 28, at index 1, the original's `Unit::think_scout+0x941`
        // and 26 `+0xaba` ours does not make. Its block 8908 parts on
        // `1/35` alone, eight keys — among them its group, ours 65 and the
        // original 79, and its order's kind, ours 10 and the original 3.
        // **Item 1214 closed it** (`docs/COMBAT.md` §86): `1/35` is the
        // computer's Caravel, idle on the sea at (7392, 480), and ours'
        // idle search took the human's building `0/2004` at (4992, 4992),
        // inland and out of its range — `Object::check_target`'s head
        // refuses a candidate in another region out of range, so the
        // original's think went on to `think_scout` and `EXPLORE_TO` (504,
        // 504) in a new group 79. Both rows agree now, and `1/35` parts no
        // key in the window.
        pin_eq!(
            row(1, 35, "group").as_deref(),
            None,
            "the word's block: `1/35` is in the original's group (item 1214)"
        );
        pin_eq!(
            row(1, 35, "order:kind").as_deref(),
            None,
            "on the original's order"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(8815, 169)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 8908)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(8815, 109)],
            "the blocks keys first part on, to the old word's block 8908"
        );
        // Item 1197: 1176 → 627, and block 8815's 504 → 183. Item 1214:
        // 627 → 195; the first key past block 8815 parts on 8947. Item
        // 1228: 195 → 190, and block 8815's 183 → 178, who=1's
        // Merchant Fleets counted (`docs/TRANSPORT.md` §16).
        // Item 1291 took 2: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 180.
        pin_eq!(w.firsts.len(), 117, "every key parted on run445");
    }

    /// **The second pair's East Indies word 10183, widened whole** (item
    /// 1214): run462 is run346's game at run445's detail over blocks
    /// 10178..10434, walked from run346's own start — every dumped record
    /// on the word's block 10184 and the six before it, both directions.
    /// **Since item 1228 the word is 10185**, block 10186, inside the same
    /// window (8 blocks after its first and 248 before its last).
    #[test]
    fn run462_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run462",
            "gamelog-run462-islands-toughest-10183.txt",
            WIDENING_SECOND_EAST_INDIES_10183,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run462 whole: blocks 10178..10434");
        pin!(
            w.missing.is_empty(),
            "run462 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 10183** (item 1214; no mechanism is named): ours 24
        // draws against 9, at index 0, ours' `Leader::create_units+0x642`
        // where the original spends `Guy::set_anim+0x97a < do_cast`. The
        // first block stands on 182 keys, among them who=1's Merchant
        // Fleets — `num_units[268]` ours 0 against 1, standing since
        // run445's 8815 (0 against 2) — and the make list parts from 10181.
        // Its block 10184 parts on nine keys, the make list's head among
        // them.
        //
        // **The word 10185** (item 1228, `docs/TRANSPORT.md` §16): the
        // Merchant Fleet is counted, so who=1's `control`, `effective_pop`
        // and `num_units[268]` agree from the first block, and the second
        // `create_units` pass on 10183 fails the population gate on both
        // sides. Ours 9 draws against 10 on 10185, at index 1: the
        // original's second `Leader::use_market`, then its Senate. On
        // 10185 the Senate offers stand at a quarter of the original's —
        // `check_income` finds it unaffordable here — and the word's block
        // 10186 holds the Senate's foundation `1/2026` on the dump alone.
        for key in [
            "leader:num_units[268]",
            "leader:control",
            "leader:effective_pop",
            "leader:MAKE[0].t",
            "leader:active",
        ] {
            pin_eq!(
                row(1, -1, key),
                None,
                "who=1's {key} agrees on the window (item 1228)"
            );
        }
        //
        // **The word 10985** (item 1243, `docs/TECH.md` "The queue loop"): a
        // unit research gains before it unqueues, so who=1's Pikemen
        // research at `1/2020` on 9143 leaves `num_queued[84]` at 0 with the
        // re-targeted Hoplites entry in the queue, not at 1. **The move's
        // value diff, here** — each key agreeing now, and each read on the
        // block it parted on under 1228's tree: on 10178, who=1's
        // `num_queued[84]` ours 1 against 0, `bucket[0:food]` 125 against
        // 133 and `bucket[4:metal]` 107 against 115, and `1/2020`'s
        // `queue[0].cost` 86/66 against 78/58 (the second Pikemen priced a
        // ramp step dearer); on 10183 `bucket[1:timber]` 48 against 56; on
        // 10185 `MAKE[0].val` 1,200,000 against 4,800,000 and `MAKE[1].t`
        // −1 against 438 (the second Senate); on 10186 `bucket[2:wealth]`
        // 57 against 7 and `1/2026`, the Senate's foundation, the dump's
        // alone. Frame 10185's draws went 9 against 10 → agreeing. The
        // word left the window: 10985, widened on run480.
        for (who, o, key) in [
            (1, -1, "leader:num_queued[84]"),
            (1, -1, "leader:bucket[0:food]"),
            (1, -1, "leader:bucket[4:metal]"),
            (1, 2020, "queue:queue[0].cost[0]"),
            (1, 2020, "queue:queue[0].cost[1]"),
            (1, -1, "leader:bucket[1:timber]"),
            (1, -1, "leader:MAKE[0].val"),
            (1, -1, "leader:MAKE[1].t"),
            (1, -1, "leader:bucket[2:wealth]"),
            (1, 2026, "build:unlinked"),
        ] {
            pin_eq!(
                row(who, o, key),
                None,
                "{who}/{o}'s {key} agrees on the window (item 1243)"
            );
        }
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // The blocks to 10186, 1228's word's block: the word has left the
        // window, so the count stops where it stood.
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 10_186)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(10178, 107), (10184, 2)],
            "the blocks keys first part on, to 10186"
        );
        // Item 1228: 1055 → 1398; the window now walks past 10185, and
        // who=1's make list and its Senate part from there. Item 1243:
        // 1398 → 198, the word past the window.
        // Item 1291 took 3: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 187.
        pin_eq!(w.firsts.len(), 119, "every key parted on run462");
    }

    /// **The second pair's East Indies word 10985, widened whole** (item
    /// 1243): run480 is run346's game at run462's detail over blocks
    /// 10980..11236, walked from run346's own start — six blocks before
    /// the word's block 10986 and 250 past it.
    #[test]
    fn run480_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run480",
            "gamelog-run480-islands-toughest-10985.txt",
            WIDENING_SECOND_EAST_INDIES_10985,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run480 whole: blocks 10980..11236");
        pin!(
            w.missing.is_empty(),
            "run480 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 10985** (item 1243) **→ 11328 on item 1264**
        // (`docs/TECH.md` step 8): who=1's Tower `1/2014` became a Keep
        // when the Keep was gained (before run425's 7377), and this crate
        // kept a Tower. `create_buildings` then read no Keep, and on 10984
        // offered a first one at 900,000, which took `MAKE[1]` ahead of the
        // Pikemen. **The move's value diff (the word's block before, here):**
        // on 10985 who=1's `MAKE[1].t` ours 440 against 134 → agreeing,
        // `MAKE[1].val` 900000 against 611022 → agreeing, and the eight
        // make-list rows that part there → none. Frame 10985's draws went
        // 10 against 11 → agreeing. The new word 11328 is past this window.
        pin_eq!(
            row(1, -1, "leader:MAKE[1].t"),
            None,
            "the block before the word's: the Pikemen, no Keep above them"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[1].val"),
            None,
            "the block before the word's: its offer"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 10_986)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(10980, 117), (10981, 1)],
            "the blocks keys first part on, to the old word's"
        );
        // Item 1264 took the Keep's offer and everything downstream of it
        // (1490 → 220).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 211.
        pin_eq!(w.firsts.len(), 136, "every key parted on run480");
    }

    /// **run490 — the second pair's East Indies word 11328, widened whole**
    /// (item 1264): run480's detail over blocks 11323..11579, walked from
    /// run346's start with the group record and the attack order's row.
    #[test]
    fn run490_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run490",
            "gamelog-run490-islands-toughest-11328.txt",
            WIDENING_SECOND_EAST_INDIES_11328,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run490 whole: blocks 11323..11579");
        pin!(
            w.missing.is_empty(),
            "run490 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 11328** (item 1264): on its block 11329 no leader or
        // city row parted, and 115 figure rows did — 65 units' `g.gpiece[0]`
        // the original's 2112 (0x840) above ours, and fifteen `dest_angle`s.
        // **Item 1281 closed them** (`docs/TECH.md`, "The piece moves with
        // the age"): 115 → 16, and the sixteen are the merchants `1/19` and
        // `1/59`'s over-time piece (`docs/ANIM.md` §3.4). The standing
        // floor lost eight `dest_angle` rows an earlier age's snap wrote.
        pin_eq!(
            row(1, 1, "g.gpiece[0]"),
            None,
            "the age's block: a figure's graphic piece agrees"
        );
        pin_eq!(
            row(1, 19, "g.gpiece[0]").as_deref(),
            Some("11329: ours 2123 theirs 50689"),
            "the age's block: a merchant's over-time piece"
        );
        // **The word 11549** (item 1281): ours 11 draws against 4, at
        // index 0, ours' `Unit::think_scout+0x941` where the original
        // spends `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`. On its block
        // 11550 the Caravel `1/35`'s order kind and path parted, ours 3
        // (`EXPLORE_TO`) against 2 (`ATTACK_TO`). **Item 1297 closed it**
        // (`docs/SCOUT.md` §13 item 1b): on frame 11523 its region scan
        // found nothing, and a sea unit's tail joins an army
        // (`think_scout` at `5f6db5`) — army 3's group 72, 4 → 5 members,
        // walking to its first on an `ATTACK_TO`. Blocks 11524 (11 keys),
        // 11525 (17) and 11550 (23) agree; the word is 11637, past run490.
        pin_eq!(
            row(1, 35, "order:kind"),
            None,
            "the word's block: the Caravel's order agrees"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // **Item 1305** wired the army's generals' turn (`docs/AI.md`
        // §99.13): the General `1/98`'s `idle`, `order:length` and
        // `orders.len`, ours 5, 0, 0 against 0, 1, 1 on 11411 → agreeing
        // to 11538; they part on 11539 (ours 3, 0, 0 against 0, 1, 1).
        // **Item 1302** closes 11539: the tick's `ATTACK_TO` on 11508
        // hands the first cast's craft back and `Unit::work` restarts its
        // clock, so the second cast is laid on 11538 on both sides
        // (`docs/AI.md` §99.14); 11539 is `1/79`'s `dest_angle` alone.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SECOND_WORD_EAST_INDIES + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [
                (11323, 115),
                (11329, 16),
                (11356, 3),
                (11401, 1),
                (11411, 2),
                (11413, 6),
                (11414, 2),
                (11445, 2),
                (11513, 1),
                (11539, 1),
                (11542, 1)
            ],
            "the blocks keys first part on, to the window's end"
        );
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1302 took three on 11411: `1/98`'s Create Decoys, laid by
        // army 4's spellcaster turn — its `spell_time` 1 and `mana_burn`
        // 1000 (`docs/AI.md` §99.14).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 226.
        pin_eq!(
            w.firsts.len(),
            150,
            "every key parted on run490 (1311 before item 1281, 382 before 1297, 229 before 1302)"
        );
    }

    /// **run506 — the second pair's East Indies word 11637, widened whole**
    /// (item 1297): run490's detail over blocks 11632..11888, walked from
    /// run346's start with the group record and the attack order's row.
    #[test]
    fn run506_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run506",
            "gamelog-run506-islands-toughest-11637.txt",
            WIDENING_SECOND_EAST_INDIES_11637,
            // 11637's own block: the group record the copies join.
            &[11_638],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run506 whole: blocks 11632..11888");
        pin!(
            w.missing.is_empty(),
            "run506 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 11637** (item 1297), closed by item 1302: the
        // eighteen births were six decoy squads of who=1's General `1/98`
        // (`docs/AI.md` §99.14). On 11638 group 70 lists 37 on both sides
        // and every copy stands where the original's does; what is left on
        // the block is the eighteen copies' standing birth `form` (0
        // against −1, the added-unit family) and `1/93`'s `path_recursion`
        // (1 against 0), the one copy born into a reused slot.
        pin_eq!(
            row(1, -3, "group:70.num"),
            None,
            "the word's block: the group the copies join agrees"
        );
        pin_eq!(
            row(1, 104, "unlinked"),
            None,
            "the word's block: the copies are this crate's too"
        );
        pin_eq!(
            row(1, 93, "path_recursion").as_deref(),
            Some("11638: ours 0 theirs 1"),
            "the copy in a reused slot"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(11632, 194), (11638, 19)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 11_638)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(11632, 126), (11638, 1)],
            "the blocks keys first part on, to 11637's"
        );
        // Item 1302 took 1151: the six squads' records, the group's list,
        // and what followed them (1375 before).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 224.
        pin_eq!(w.firsts.len(), 137, "every key parted on run506");
    }

    /// **run508 — the second pair's East Indies word 12582, widened whole**
    /// (item 1302): run506's detail over blocks 12577..12833, walked from
    /// run346's start with the group record and the attack order's row.
    #[test]
    fn run508_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run508",
            "gamelog-run508-islands-toughest-12582.txt",
            WIDENING_SECOND_EAST_INDIES_12582,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run508 whole: blocks 12577..12833");
        pin!(
            w.missing.is_empty(),
            "run508 carries every key: {:?}",
            w.missing
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **The word 12582** (item 1302; no mechanism is named): ours 9
        // draws against 10, at index 4, where the original spends
        // `Leader::make_stuff+0x63d`. The first rows before it are on
        // block 12581: who=1's `MAKE[5]` holds a Citizen order (`t` 50,
        // `cat` 5, `num` 4, `city` 2, `escrow` 1) the dump alone has, and
        // `MAKE[0]`, `[2]` and `[6]`'s `city` read 0 against ours' 1.
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was Some("12581: ours -1 theirs 50").
        pin_eq!(
            row(1, -1, "leader:MAKE[5].t").as_deref(),
            None,
            "the block before the word's: a make slot the dump alone fills"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        // Item 1326 re-pinned on the tree merged with 1318's: was [(12577, 210), (12582, 1)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SECOND_WORD_EAST_INDIES + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(12577, 130), (12582, 1), (12601, 1), (12763, 1), (12809, 1)],
            "the blocks keys first part on, to the word's"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 1817.
        pin_eq!(w.firsts.len(), 134, "every key parted on run508");
    }

    /// **run523 — the second pair's East Indies word 13385, widened whole**
    /// (item 1326): run508's detail over blocks 13380..13636, walked from
    /// run346's start.
    #[test]
    fn run523_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run523",
            "gamelog-run523-islands-toughest-13385.txt",
            WIDENING_SECOND_EAST_INDIES_13385,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run523 whole: blocks 13380..13636");
        pin!(
            w.missing.is_empty(),
            "run523 carries every key: {:?}",
            w.missing
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **The word 13385** (item 1326): ours 4 draws against 54, at
        // index 0, where the original's step-11 `make_stuff` spends
        // `Leader::use_market+0x1ed` and places a Farm. The first rows
        // were two blocks earlier, on 13383, step 8's purchase (frame
        // 13382): who=1's Trebuchet order (`num` 2) queued two here and
        // one there, the original's at `1/2028` for 76 timber and 76 metal
        // where ours' first cost 66 and its second 85. **Item 1341**
        // (`docs/AI.md` §56.5): `get_cost`'s bump loop — the Bombard's
        // research queued at `1/2024` makes a Trebuchet's base 8, so 80 ×
        // 95/100 = 76, and the second, 100 × 95/100 = 95, is past the 89
        // timber left. Both rows agree, and blocks 13383..13385 with them.
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        pin_eq!(
            row(1, -1, "leader:num_queued[216]").as_deref(),
            None,
            "the Trebuchets queued on step 8's purchase"
        );
        pin_eq!(
            row(1, 2028, "queue:queue[0].cost[0]").as_deref(),
            None,
            "the first Trebuchet's price"
        );
        // Item 1341: [(13380, 127), (13383, 7), (13384, 1), (13385, 6), (13386, 35)] before the bump loop (on the tree merged with 1330's).
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 13_386)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(13380, 126), (13386, 2)],
            "the blocks keys first part on, to the word 13385's"
        );
        // Item 1326 re-pinned on the tree merged with 1318's: was 1103.
        // Item 1341, the bump loop: 1015 → 138 (on the tree merged with
        // 1330's, whose birth `form` took 1102 → 1015).
        pin_eq!(w.firsts.len(), 135, "every key parted on run523");
    }

    /// **run535 — the second pair's East Indies word 14141, widened whole**
    /// (item 1341): run523's detail over blocks 14136..14392, walked from
    /// run346's start.
    #[test]
    fn run535_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run535",
            "gamelog-run535-islands-toughest-14141.txt",
            WIDENING_SECOND_EAST_INDIES_14141,
            &[14_142],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run535 whole: blocks 14136..14392");
        pin!(
            w.missing.is_empty(),
            "run535 carries every key: {:?}",
            w.missing
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **The word 14141** (item 1341) was who=1's decoys outliving
        // their age: `1/104`..`1/120` stood ours alone on 14137 and `1/93`
        // on 14138. **Item 1351 closes a decoy at `(general_upgrade + 2) ×
        // DECOY_TIME / 2`** (`docs/GOLDEN.md` §48), and both rows agree:
        // block 14136 stands on its 138 keys and nothing parts on the
        // word's own block 14142. The first row past it is the leader's
        // `production_step` on 14201.
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        pin_eq!(
            row(1, 104, "extra"),
            None,
            "the first of the seventeen copies closes on both sides"
        );
        pin_eq!(row(1, 93, "extra"), None, "and the one a block later");
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 14_142)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(14136, 135)],
            "the blocks keys first part on, to the word 14141's"
        );
        // Measured on the tree merged with 1330's (1818 before it); item
        // 1351, the decoy's close: 1730 → 163.
        pin_eq!(w.firsts.len(), 160, "every key parted on run535");
    }

    /// **run544 — the second pair's East Indies word 15862, widened whole**
    /// (item 1351): run535's detail over blocks 15857..16113, walked from
    /// run346's start. Item 1362 moved the word to 15883, inside it.
    #[test]
    fn run544_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run544",
            "gamelog-run544-islands-toughest-15862.txt",
            WIDENING_SECOND_EAST_INDIES_15862,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run544 whole: blocks 15857..16113");
        pin!(
            w.missing.is_empty(),
            "run544 carries every key: {:?}",
            w.missing
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **The word 15862** (item 1351) was ours four `find_target`
        // scores on who=1's navy, army 3, retargeting from its own
        // Newcastle. **Item 1362 builds the escort** (`Armies::send_navy`,
        // `docs/TRANSPORT.md` §8.3): army 0's take of who=0's Napata on
        // 15612 hands the navy that target, and on 15862 it keeps it. The
        // word's own block 15863 went 88 → 0: `1/30` and the four others of
        // group 72 hold the original's one order to (33240, 25800). Block
        // 15857 stands on 140 keys, and who=1's group 70 on 15859 (the
        // original's fifteen slots and ours none, `ox` 0 against −1) beside
        // group 65's `held` [158, 159, 160] ours alone stand as they did.
        // Army 0's group 71 parts on 15869 — every member's order
        // `group.id` — and on 15882 `1/132`'s figures, ahead of the word
        // 15883 (block 15884). **Item 1370 builds the pack before a march**
        // (`Unit::work`'s pack arm, `docs/ORDERS.md` §6.9.2): the Bombard
        // `1/132` stands deployed when army 0's march reaches it on 15868,
        // and the original packs it for 80 frames first. Group 71's `speed`
        // (23 against 25) and every `1/132` row leave; 15869 keeps the 47
        // `group.id` rows, the declared stand-in that does not score. Who=1's
        // `caras` on 15924 (ours 3, theirs 2) and `MAKE[1].val` on the new
        // word's frame 15985 stood before the item and stand; the word's
        // own block 15986 parts on 47 keys: `1/2047` placed at (35904,
        // 25728) against (39744, 21888), and `1/88` and `1/122` each on the
        // other's order. **Item 1377 builds the Tower line in the
        // placement** (`docs/AI.md` §100): a Keep is `is(0x1b7, 0)`, so its
        // spiral starts at the anchor, its friends are squared and two
        // farms put it on (51, 28); 15986 goes 47 → 0, who=1's `defense`
        // with it (`Build::init`'s `+1`). Who=0's `production_step` on
        // 16001 is the human the crate never steps (`docs/AI.md` §23.1); the
        // new word's block 16010 parts on 23 keys, all the Supply Wagon
        // `1/153`'s: its figures and its move's `pause`, 14 against 15.
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        pin_eq!(
            row(1, -3, "group:70.ox").as_deref(),
            Some("15859: ours -1 theirs 0"),
            "the first row past the standing block"
        );
        pin_eq!(
            row(1, 30, "orders.len").as_deref(),
            None,
            "the navy's one order on 15863 agrees (item 1362)"
        );
        pin_eq!(
            row(1, 132, "pos").as_deref(),
            None,
            "the Bombard's pack agrees to the window's end (item 1370)"
        );
        pin_eq!(
            row(1, -3, "group:71.speed").as_deref(),
            None,
            "the march's speed agrees: the packing Bombard reports none"
        );
        pin_eq!(
            row(1, 2047, "build:x_internal").as_deref(),
            None,
            "the Keep's site on (51, 28) agrees: the Tower line (item 1377)"
        );
        pin_eq!(
            row(1, 88, "order:kind").as_deref(),
            None,
            "the Keep's builder agrees: `1/122` is called, `1/88` gathers"
        );
        pin_eq!(
            row(1, -1, "leader:defense").as_deref(),
            None,
            "who=1's `defense` counts the placed Keep (`Build::init`, item 1377)"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SECOND_WORD_EAST_INDIES + 1)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [
                (15857, 139),
                (15859, 49),
                (15869, 47),
                (15924, 1),
                (15985, 1),
                (16001, 1),
                (16010, 23)
            ],
            "the blocks keys first part on, to the word's"
        );
        // Item 1362, the escort: 1386 → 1380. Item 1370, the pack: 1380 → 982.
        // Item 1377, the Keep's site: 982 → 662.
        pin_eq!(w.firsts.len(), 661, "every key parted on run544");
    }

    /// **The gap 6573..6603 of the second pair's East Indies, walked whole**
    /// (item 1156): run421 is run346's game at run420's detail over blocks
    /// 6567..6610, walked from run346's own start — the frames between
    /// run419's last block and run420's first, where the word 6609's `1/28`
    /// first parted.
    #[test]
    fn run421_s_gap_is_walked_whole() {
        let _pins = Pins::hold();
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run421",
            "gamelog-run421-islands-toughest-gap6573.txt",
            GAP_SECOND_EAST_INDIES_6573,
            &[6_577],
            true,
            true,
        ) else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, r)) in &w.firsts {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 44, "run421 whole: blocks 6567..6610");
        pin!(
            w.missing.is_empty(),
            "run421 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word 6609's first parting** (item 1156, `docs/SCOUT.md`
        // §8.1): the AI citizen `1/28`, idle at (29568,24192) on 6576,
        // takes its region scan's cell. The original's block 6577 holds
        // `orders_x/y` (26616,22776), its leg to (26592,22752) — cell
        // (34,29), 128 — where ours sent it to (28920,25080), cell (37,32),
        // until `find_unit_ordered` measured `1/22` by its walk's end
        // (27384,25080) rather than its body. All of it agrees, and so does
        // the goody look's re-aim on 6588.
        for what in [
            "orders_x",
            "orders_y",
            "pos",
            "order:move.angle",
            "path:length",
        ] {
            pin_eq!(row(1, 28, what), None, "`1/28`'s {what}");
        }
        // What is left on the gap is standing or known: the first block's
        // 258 keys (263 until item 1174 fed who=1's `escrow`, `docs/AI.md`
        // §98) (run420's first block held the same families); groups'
        // `speed`/`form`, which this crate writes on no one-member group;
        // the barge identities `1/36`/`1/38` (6574, parked since item 1143);
        // and `1/42`'s `path[16].flags` 4 against 0 on 6576.
        pin_eq!(
            row(1, 42, "path[16].flags").as_deref(),
            Some("6576: PathField { slot: 16, field: \"flags\", ours: 4, theirs: 0 }"),
            "a flag on a slot the original leaves clear"
        );
        let mut by: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // Item 1291 took rows: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.into_iter().collect::<Vec<_>>(),
            [
                (6567, 102),
                (6570, 2),
                (6576, 4),
                (6577, 2),
                (6582, 1),
                (6587, 4),
                (6601, 1)
            ],
            "the blocks keys first part on, the whole window"
        );
        // Item 1197: 277 → 175, the dead transports' numbers held
        // (`docs/COMBAT.md` §59.3). Item 1228: 175 → 171, the first
        // block's 158 → 155 and 6577's 4 → 3, who=1's Merchant Fleets counted (`docs/TRANSPORT.md` §16).
        // Item 1291 took one: a landed passenger's `mirror` agrees — `set_new_location` keeps the unit's own angle, and `set_angle`'s flip reads it (`docs/TRANSPORT.md` §17).
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 170.
        pin_eq!(w.firsts.len(), 116, "every key parted on run421");
    }

    /// **The second pair's Great Lakes word, 4555, widened whole** (item
    /// 989): run356 is run347's game at run355's detail over blocks
    /// 4550..4806, walked from run347's start. The word's frame writes
    /// block 4556. **Since item 997 the word is 4593**, block 4594, inside
    /// the same window (44 blocks after its first and 212 before its last),
    /// so this capture is its widening too. **Since item 1002 the word is
    /// 4605**, block 4606, inside it again (56 blocks after its first and
    /// 200 before its last). **Since item 1012 the word is 4618**, block
    /// 4619 (69 blocks after its first and 187 before its last). **Since
    /// item 1014 the word is 4673**, block 4674 (124 blocks after its first
    /// and 132 before its last). **Since item 1023 the word is 4688**, block
    /// 4689 (139 blocks after its first and 117 before its last). **Since
    /// item 1034 the word is 4781**, block 4782 (232 blocks after its first
    /// and 24 before its last). **Since item 1040 the word is 4846**, block
    /// 4847, past this window: run373 is its widening.
    #[test]
    fn run356_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run356",
            &[(
                "gamelog-run356-greatlakes-toughest-4555.txt",
                WIDENING_SECOND_GREAT_LAKES_4555.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_4555,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        pin_eq!(w.blocks, 257, "run356 whole: blocks 4550..4806");
        pin!(
            w.missing.is_empty(),
            "run356 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The old word's block, 4555, agrees** (item 997, `docs/COMBAT.md`
        // §64): `1/21`'s attack-move no longer takes the human's unarmed
        // Woodcutter's Camp on 4554, so its `ATTACK_TO` stands on both
        // sides until the army's attack on 4605, and since item 1012 its
        // order agrees through that too; it first parted again on 4761
        // until item 1028, and since then not in the window.
        pin_eq!(
            row(1, 21, "order:kind"),
            None,
            "the unit's order, which parted on 4555 until item 997"
        );
        // **The old word, 4593, agrees** (item 1002, `docs/COMBAT.md`
        // §65): `1/26`'s chase on the human's city `0/2000` stops at
        // (5079, 31628) on block 4592 on both sides — a building target is
        // asked without the `0x90` margin — and holds its `ATTACK` there
        // until block 4605, where both sides drop it; since item 1012 the
        // army's attack puts it back on the city on 4605 on both sides, and
        // neither its order nor its position parts again in the window.
        pin_eq!(
            row(1, 26, "order:kind"),
            None,
            "the unit's order, which parted on 4592 until item 1002"
        );
        pin_eq!(row(1, 26, "pos"), None, "and where it stands");
        // **The old word, 4605, agrees** (item 1012, `docs/COMBAT.md` §66):
        // the army group's attack-move looks on 4605 (`1/15`, `(4605 + 15)
        // % 15 == 0`), finds the human's city, and hands it to the whole
        // group — `Group::action_attack(…, QUEUE_FIRST, 4)` — whose retarget
        // takes a building with the word 2, and `1/17`'s ring starts
        // mid-face. Every member holds the city's `ATTACK` over the
        // re-issued `GROUP_ATTACK_TO` on block 4606 on both sides; `1/9`
        // first parted on 4771 until item 1014, and since then its order
        // agrees through the window.
        pin_eq!(
            row(1, 9, "order:kind"),
            None,
            "an army member's order, which parted on 4606 until item 1012"
        );
        // **The old word, 4618, agrees** (item 1014, `docs/SCOUT.md` §8.3):
        // the AI scout `1/0` chooses its explore target on tick 4506 with
        // the human's cells undoubled — `treaties[0] & 3` is the met bit,
        // set on 4456 — and takes cell (3, 42), (2808, 32760), on both
        // sides, where ours took (2, 40), (2040, 31224), arrived on 4618
        // and idled. Its order agrees from block 4550 to its next explore
        // target on 4737; `1/0`'s group and `1/7`'s gather `wait`, which
        // parted on 4619, no longer part in the window. That target read
        // 1272 here from item 1023 and 2808 again from item 1028, against
        // the original's 504; **since item 1034 it agrees**, and the
        // scout's order agrees through the window. Which read of the city
        // moved it is not established: the fix changed only the city's
        // three flags and its heal (`docs/COMBAT.md` §69).
        pin_eq!(
            row(1, 0, "order:move.x"),
            None,
            "the scout's target, which parted on 4550 until item 1014 and on \
             4737 until item 1034"
        );
        pin_eq!(
            row(1, 0, "group"),
            None,
            "the scout's group, 4619 until 1014"
        );
        pin_eq!(
            row(1, 7, "order:gather.wait"),
            None,
            "the gatherer's wait, 4619 until item 1014"
        );
        // **The old word, 4673, agrees** (item 1023, `docs/COMBAT.md` §67):
        // on 4616, the review's phase for `1/24` (`(4616 + 24) % 16 == 0`),
        // its target, the citizen `0/3`, is walking away and out of reach
        // of the chase spot, so `check_target_path` re-aims: block 4617
        // prints the move at (4104, 31512) on both sides, where ours kept
        // (4200, 31608). `1/24`'s order (kind 10 against 1 on 4673) and
        // `stopped` (4674) no longer part; its rows agree from 4605 to 4688.
        // Its order parted again on 4722 until item 1028, and since then
        // not in the window.
        pin_eq!(
            row(1, 24, "order:kind"),
            None,
            "the chaser's order, which parted on 4673 until item 1023"
        );
        // **The old word, 4688, agrees** (item 1028, `docs/COMBAT.md` §68):
        // `1/24` is a computer's raider (stance 3), and `compare_target`'s
        // RAID arm scores the citizen `0/3` at 9800 and the human's scout
        // `0/0` at 15, as run368's packet does on logger frame 4688. So its
        // one-in-five re-search keeps `0/3`, and it strikes: `recharging`
        // (0 against 33 until 1028) no longer parts on block 4689. Its walk
        // spot parted there, (3763, 31600) against the cell centre (3768,
        // 31608), **until item 1099** (`docs/COMBAT.md` §80): the re-search
        // is `find_new_target`, which kills the attack and adds it fresh,
        // and `fight` snaps the unit to its cell centre under the new order.
        pin_eq!(
            row(1, 24, "recharging"),
            None,
            "the chaser's strike, on 4689 until item 1028"
        );
        pin_eq!(
            row(1, 24, "orders_x"),
            None,
            "and its spot, which parted on 4617 until item 1023 and on 4689 \
             until item 1099"
        );
        // **The old word, 4690, agrees** (item 1034, `docs/COMBAT.md` §69):
        // `1/26`'s first strike on the human's city on 4657 sets its
        // `city_flags` `0x2 | 0x4 | 0x8` on both sides, and the city no
        // longer heals the wound off (ours 1/0 against 2/5 on 4661 until
        // 1034), so 4690's strike is not a first wound and throws no
        // `take_damage+0xe1`. The three bits now agree through the window,
        // `0x4`'s clear on 4801 (`(4800 + 2000) % 200 == 0`) included.
        for bit in ["0x2", "0x4", "0x8"] {
            pin_eq!(
                row(0, 2000, &format!("city:city_flags[{bit}]")),
                None,
                "the human city's {bit}, which 1034 carries"
            );
        }
        // **A Hoplite on the city's north face strikes due south**
        // (item 1040, `docs/COMBAT.md` §70.6): `1/22`'s facing stood parted
        // from 4758 (ours the centre's bearing, 0x959a0000, against
        // 0x80000000) until the side arm.
        pin_eq!(
            row(1, 22, "g.angle[0]"),
            None,
            "the Hoplite's facing, which parted on 4758 until item 1040"
        );
        // What the heal had hidden since 4661: `1/26`'s strikes from the
        // second on landed here a frame after the original's (4689, 4716,
        // 4722 there; 4690, 4717, 4723 here) until item 1040: its stones
        // left the Slinger's own square, not its bay (`launch::BAYS`, piece
        // 32, `docs/COMBAT.md` §70), and flew a frame long across a
        // boundary. Now every strike lands on the original's frame.
        pin_eq!(
            row(0, 2000, "build:damage"),
            None,
            "the city's damage, one strike behind on 4690 until item 1040"
        );
        // **The citizen heals on its own ground** (item 1072,
        // `docs/COMBAT.md` §72): `0/3`, struck on 4710 to 3/5, heals a
        // point and its fraction on frame 4722 (`(3 + 4722) % 45 == 0`) and
        // again on 4767, 2/0 and 1/0 on blocks 4723 and 4768 on both
        // sides, where ours held 3/5 (4723's row, ours 3 against 2, until
        // 1072).
        for what in ["hits:damage", "hits:damage_frac", "hits_left"] {
            pin_eq!(
                row(0, 3, what),
                None,
                "the citizen's {what}, which parted on 4723 until item 1072"
            );
        }
        // **The old word, 4781, agrees** (item 1040, `docs/COMBAT.md` §70):
        // `1/24`'s stone at the citizen `0/4`, launched on 4775, leaves the
        // Slinger's bay and flies 5 frames on both sides, so `0/4` is struck
        // on 4779 on both; and a citizen walking to its drop site under its
        // `GATHER` is not on duty (`Unit::on_duty@005fff70`), so it does not
        // turn on the Slinger (`600863`): no `ATTACK` on 4780, no
        // `fight+0x9b0` on 4781. None of `0/4`'s rows parts in the window.
        for what in ["damage_frame", "order:kind", "pos"] {
            pin_eq!(
                row(0, 4, what),
                None,
                "the citizen's {what}, which parted on 4780 until item 1040"
            );
        }
        // **The group record and the attack order's row** (item 1061,
        // `docs/GROUPS.md` §33): three rows stand from the window's first
        // block — army 0's group (slot 65) `role`, ours 0 against the
        // original's `LAND | MILITARY | …` word, and the Town Center's
        // building group (slot 64) `ox`/`oy`, ours −1 against 0 — and the
        // army group's `curr` parts inside it on 4600, with `1/19`'s
        // heading on the same block. The `ATTACKORDER`'s own row on 4753,
        // `1/24`'s `ever_in_range` 1 against 0 and `new_ord` 0 against 1,
        // parted **until item 1099** (`docs/COMBAT.md` §80): the one-in-five
        // re-search adds its order fresh, as the original's does (parked
        // 1073).
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4550, 70), (4585, 1), (4598, 1)],
            "the blocks keys first part on, the first three ((4550, 76) until \
             item 1061 compared the group record; 86, 18 and 1 on 4550, 4575 \
             and 4585 until item 1014, the scout's ten and eighteen)"
        );
        for (who, o, what, want) in [
            (1, -3, "group:65.role", "4550: ours 0 theirs 1379331"),
            (1, -3, "group:64.ox", "4550: ours -1 theirs 0"),
        ] {
            pin_eq!(
                row(who, o, what).as_deref(),
                Some(want),
                "{who}/{o} {what}, the record item 1061 compares"
            );
        }
        // **Item 1206 closed group 65's `curr[0]`** (4600: ours (190, 414)
        // against (192, 414)): a member that takes the formation over turns
        // it through `update_positions`' waypoint arm, not by its heading
        // (`docs/GROUPS.md` §6.8).
        pin_eq!(
            row(1, -3, "group:65.curr[0]"),
            None,
            "1/-3 group:65.curr[0] agrees on 4600"
        );
        for what in ["attack[0].ever_in_range", "attack[0].new_ord"] {
            pin_eq!(
                row(1, 24, what),
                None,
                "1/24 {what}, which parted on 4753 until item 1099"
            );
        }
    }

    /// **The second pair's Great Lakes word, 4846, widened whole** (item
    /// 1040): run373 is run347's game at run356's detail over blocks
    /// 4841..5097, walked from run347's start. The word's frame writes
    /// block 4847, six blocks after the window's first. **Within the same
    /// item the word moved to 4852 and then 4877**, block 4878, inside the
    /// window (37 blocks after its first and 219 before its last). **Item
    /// 1052 moved it to 4924**, block 4925, inside the window too (84
    /// blocks after its first and 172 before its last), **item 1061
    /// to 4978**, block 4979 (138 after its first and 118 before its
    /// last), **item 1072 to 5042**, block 5043 (202 and 54), **item
    /// 1081 to 5066**, block 5067 (226 and 30), **item 1086 to 5075**,
    /// block 5076 (235 after its first and 21 before its last), and **item
    /// 1074 to 5105**, block 5106, nine past its last: run396 widens it
    /// (and item 1089 to 5161, run396's block 5162).
    /// This window keeps what it has always held, every old word's block.
    #[test]
    fn run373_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run373",
            &[(
                "gamelog-run373-greatlakes-toughest-4846.txt",
                WIDENING_SECOND_GREAT_LAKES_4846.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_4846,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // `RON_FIRSTS=<block>` prints every key's first parting from that
        // block on, both sides' values beside it (item 1081).
        if let Some(from) = std::env::var("RON_FIRSTS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
        {
            let mut rows: Vec<_> = w.firsts.iter().filter(|(_, (f, _))| *f >= from).collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run373 whole: blocks 4841..5097");
        pin!(
            w.missing.is_empty(),
            "run373 carries every key: {:?}",
            w.missing
        );
        let first = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| (*f, r.clone()))
        };
        // **The old word, 4846, agrees** (item 1040, `docs/COMBAT.md`
        // §70.6): the Hoplite `1/19` stands on the row above the human's
        // city and strikes it due south on both sides — a building is
        // struck square to its side — where ours turned 21° to the centre,
        // deferred the swing and struck a frame late. Its figure first
        // parts on the army's tick, 4861, until item 1052, on 4989 until
        // item 1072 (a chase that ended on `0/2`'s early death), on 5085
        // until item 1074, and not in the window since.
        pin_eq!(
            first(1, 19, "g.angle[0]").map(|(f, _)| f),
            None,
            "the Hoplite's facing, which parted on 4847 until item 1040, \
             on the army's tick, 4861, until item 1052, on 4989 until \
             item 1072 and on 5085 until item 1074"
        );
        // **The old word, 4852, agrees** (§70.7): `1/24`'s one-in-five
        // re-search on 4852 names `0/3` on both sides and freezes the
        // frame, so its attack slot holds at 32 of 33 on block 4853 and
        // wraps on 4854; its clock no longer parts, and its figure first
        // parts on the army's tick too.
        for what in ["g.cur_time[0]", "g.last_time[0]", "g.end_time[0]"] {
            pin!(
                first(1, 24, what).is_none_or(|(f, _)| f > 4_853),
                "the Slinger's {what}, which parted on 4853 until item 1040: {:?}",
                first(1, 24, what)
            );
        }
        // **The old word, 4877, agrees** (item 1052, `docs/ARMY.md` §23):
        // the army's tick on 4860 (`4860 ≡ 252 mod 256`) finds army 0
        // engaged and forms it a cell **behind** its point, `muster_angle −
        // 0x80000000`, before `do_forming` moves it again. The leader
        // `1/12`'s first move hands the second's `get_loc` a start in the
        // original's cell (3996, 30614), and the group's chain plans the
        // original's eight legs, not nine. So on block 4861 every member
        // walks its `ATTACK_TO` on both sides: `1/13` at (3936, 31125)
        // toward (4588, 31018), `1/16` toward (5007, 30915), `1/12` with
        // its move under way, where ours stood `1/12`–`1/14` a frame
        // (`dest` 0) and took a leg 768 west for `1/16`–`1/18`.
        for (o, what) in [
            (12, "order:move.dest"),
            (13, "order:move.dest"),
            (14, "order:move.dest"),
            (13, "pos"),
            (16, "order:move.dest_x"),
            (17, "order:move.dest_x"),
            (18, "order:move.dest_x"),
            (16, "path:length"),
        ] {
            // Past item 1061's word's block, 4979: `1/13`'s move now
            // parts on 5012 (item 1072).
            pin!(
                first(1, o, what).is_none_or(|(f, _)| f > 4_979),
                "1/{o}'s {what}, which parted on the army's tick, 4861, until \
                 item 1052: {:?}",
                first(1, o, what)
            );
        }
        // **The old word, 4924, agrees** (item 1061, `docs/COMBAT.md` §71):
        // `do_move`'s flank clause keeps `1/11`'s chase on frame 4922,
        // where its target `0/1` walks east from its heading (`e =
        // 0xd3290000`, `flanking` 2), and ends it on 4923, `0/1` turned. So
        // on block 4923 `1/11` stands at (5032, 30200) under its chase on
        // both sides, on 4924 there under a fresh `ATTACK` on `0/1`, and
        // strikes `0/2` on 4925 on both. Its first parting row moves past
        // the word.
        for (what, block) in [
            ("order:kind", 4_980),
            ("pos", 4_985),
            ("attack[0].in_range", 4_985),
        ] {
            pin!(
                first(1, 11, what).is_none_or(|(f, _)| f >= block),
                "1/11's {what}, which parted on 4923 or 4924 until item 1061: {:?}",
                first(1, 11, what)
            );
        }
        // **The old word, 4978, agrees** (item 1072, `docs/COMBAT.md`
        // §72): a citizen on its own ground heals a point and its
        // fraction every 45 frames, `(o + frame) % 45 == 0`. `0/2` healed
        // on 4813 in the gap between run356 and this window (3/5 → 2/0,
        // then 3/5 more on 4839), and on 4858, 4903 and 4948: it stands at
        // 5/5, 4/0, 3/0 and 16/0 on blocks 4841, 4859, 4904 and 4949 on
        // both sides, where ours held 6/10 from 4841 and died on 4978 at
        // 42. `0/3`, `0/4` (4841), `0/5` (4901) and `0/1` (4905) close the
        // same way. Both sides now lose `0/2` on 5000: the original's own
        // `DEATH_OBJS`, printed only in the quit block 5111, has its
        // `first_frame` 5000 (and `0/3`'s 4996), so `death:extra` on 5001
        // (4997 for `0/3`) is the capture's, not a parting.
        for what in [
            "hits:damage",
            "hits:damage_frac",
            "hits_left",
            "hits:hold_frames",
        ] {
            for o in [1, 2, 3, 4, 5] {
                pin!(
                    first(0, o, what).is_none_or(|(f, _)| f > 5_000),
                    "0/{o}'s {what}, which parted from 4841, 4901 or 4905 \
                     until item 1072: {:?}",
                    first(0, o, what)
                );
            }
        }
        pin_eq!(
            (first(0, 2, "death:extra"), first(0, 3, "death:extra")),
            (
                Some((5_001, "ours 1 theirs 0".to_string())),
                Some((4_997, "ours 1 theirs 0".to_string()))
            ),
            "the two deaths, on the original's own frames"
        );
        // **The old word, 5042, agrees** (item 1081, `docs/COMBAT.md` §74):
        // `Ammo::init`'s lead asks the target's order (`is_move`, then
        // `is_air`) and leads along its `angle` (`UnitData +0x50`). The
        // citizen `0/1` fled between two legs on 5024 — a `FLEE_TO` head
        // and no `movement.dest`, which this crate asked — so `1/11`'s
        // round came down on (6225, 28816) unled, missed on 5040, rolled
        // and punctured the ground on 5042 (`Ammo::do_damage+0xc59`,
        // `+0xc7e`). Led, it comes down on (6497, 28561) and strikes `0/1`
        // at (6260, 28466) on 5040: 2/0 → 5/5 on block 5041, 8/10 on 5045
        // and 12/4 on 5077 (`damage_frame` 5076) on both sides, where ours
        // read 2/0, 5/5 and 5/5.
        for what in [
            "hits:damage",
            "hits:damage_frac",
            "hits_left",
            "damage_frame",
        ] {
            pin_eq!(
                first(0, 1, what),
                None,
                "0/1's {what}, which parted on 5041 (5077 for damage_frame) \
                 until item 1081"
            );
        }
        // **The old word, 5066, agrees** (item 1086, `docs/COMBAT.md`
        // §76): `check_target_path`'s `is_active` jump (`5e2429`) sends a
        // dead target into the re-aim, so `1/13`'s chase on `0/2`, dead
        // since 5000, is re-aimed at the corpse on every review — 5011,
        // 5027, 5043, 5059 — on both sides: (4872, 29928) via (4872,
        // 31464), tolerance 384 and nine legs on block 5012, where ours kept
        // its detour to (4104, 31992). On 5066 both stand at (4764, 31013)
        // under the `ATTACK`, and on 5067 both take `0/4`. Its rows first
        // part on 5068, where the chase on `0/4` is aimed at (2136, 31608)
        // against ours (3384, 31464).
        for what in ["pos", "order:kind", "order:move.dest_x", "path:length"] {
            pin!(
                first(1, 13, what).is_none_or(|(f, _)| f >= 5_068),
                "1/13's {what}, which parted on 5012 (5066 for the order) \
                 until item 1086: {:?}",
                first(1, 13, what)
            );
        }
        // **The old word, 5075, agrees** (item 1074, `docs/COMBAT.md`
        // §77): `do_move`'s action block asks of its target only that the
        // slot be live (`5f7ece`–`5f7f11`), not `valid_target`. On frame
        // 5075 player 1 has lost sight of the citizen `0/1`, which `1/10`,
        // `1/11`, `1/18`, `1/19` and `1/20` all chase; ours sent the three
        // within `0x480` of their walk's goal to the dead target's arm and
        // popped their walks, and `1/20` walked into the stopped `1/18`
        // (`collide 1`, the word's draw). Now all five walk on, on both
        // sides, and nothing parts from 5073 to 5089 but `1/0`'s figure.
        // The next rows are `1/35`'s `form` on 5090 and `1/21`'s move on
        // 5091 (no mechanism is named).
        pin_eq!(first(1, 20, "collide"), None, "1/20's collide on 1/18");
        for o in [10, 11, 18, 19, 20] {
            pin_eq!(
                first(1, o, "order:kind"),
                None,
                "1/{o}'s chase, which ended on 5075 on ours alone until item 1074"
            );
        }
        pin!(
            w.firsts
                .iter()
                .filter(|(_, (f, _))| (5_073..=5_089).contains(f))
                .all(|((who, o, _), _)| (*who, *o) == (1, 0)),
            "only 1/0's figure first parts on 5073..5089"
        );
        // **`1/13`'s chase on `0/4` and `1/21`'s goal agree** (item 1089,
        // `docs/COMBAT.md` §79): on tick 5066 `1/13`'s target `0/2` was
        // dead, and `find_new_target`'s head handed it its captain `1/12`'s
        // `0/4`, where ours searched and took `0/5`. Its move parted from
        // 5068 and refused `1/21`'s spot on 5090 through
        // `find_ordered_collision`, until then.
        pin_eq!(
            (
                first(1, 35, "form").map(|(f, _)| f),
                first(1, 21, "order:move.x"),
                first(1, 13, "order:move.x"),
                first(1, 13, "tolerance"),
            ),
            // `1/35`'s birth `form` on 5090 agrees since item 1330.
            (None, None, None, None),
            "the first rows past the old word"
        );
        // **The group record and the attack order's row** (item 1061,
        // `docs/GROUPS.md` §33): three more rows stand from the window's
        // first block — army 0's `role` and the Town Center group's
        // `ox`/`oy` (run356 has all three from 4550). **The attack order's
        // row agrees since item 1099** (`docs/COMBAT.md` §80): `1/9`'s second
        // `ATTACK` (`ever_in_range` 1 against 0 and `new_ord` 0 against 1
        // from 4841) and `1/24`'s on 4853, the block after its one-in-five
        // re-search, with `in_range` too, were the re-search's order
        // re-pointed in place where the original's `find_new_target` adds it
        // fresh (parked 1073). The next rows are the leader's `peasants` and
        // `1/34`'s `form` on 4909 (no mechanism is named).
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4841, 91), (4997, 5), (5001, 2)],
            "the blocks keys first part on, the first three ((4841, 102), \
             (4909, 2) until item 1330 took the births' `form`; (4841, 104), \
             (4853, 3), (4868, 2) until item 1099 added the re-search's order \
             fresh; (4841, 116) until item 1072 healed the citizens; (4841, \
             111), (4868, 2), (4887, 2) until item 1061 compared the group \
             record and the attack order's row; (4841, 111), (4861, 121), \
             (4862, 13) until item 1052)"
        );
        for (o, what) in [
            (9, "attack[1].new_ord"),
            (9, "attack[1].ever_in_range"),
            (24, "attack[0].in_range"),
            (24, "attack[0].new_ord"),
        ] {
            pin_eq!(
                first(1, o, what),
                None,
                "1/{o}'s {what}, which parted on 4841 or 4853 until item 1099"
            );
        }
        // The citizen `0/4` stood at 6 10/16 here against 5 0/16 from
        // the window's first block until item 1072: the heal on 4811
        // (`(4 + 4811) % 45 == 0`) took it to 5/0 there, and 3/5 more on
        // 4845 makes 8/5 on both. Keys parted: 1,052 → 471, 471 → 455
        // on item 1081 (`0/1`'s wound on 5040 and what followed it), and
        // 455 → 461 on item 1086 (`1/13`'s rows from 5012 go; the chase on
        // `0/4` from 5068 and the new word's from 5075 arrive), and 461 →
        // 203 on item 1074 (the five chases on `0/1` walk on from 5075), and
        // 203 → 156 on item 1089 (`1/13`'s rows from 5067 and `1/21`'s from
        // 5091 go), and 156 → 128 on item 1099 (the re-search's fresh
        // order, and what it moved), and 128 → 117 on item 1248 (a dead
        // citizen leaves its gather chain as it dies: `0/2001`'s and
        // `0/2002`'s `gather_down`, and who=0's `income`, `resources`,
        // `bucket`, `leftover` and `gather_stamp` from 5001).
        pin_eq!(w.firsts.len(), 103, "every key parted on run373");
    }

    /// **The second pair's Great Lakes word, 5105, widened whole** (item
    /// 1074): run396 is run347's game at run373's detail over blocks
    /// 5100..5356, walked from run347's start. The word's frame writes block
    /// 5106, six blocks after the window's first. **Item 1089 moved it to
    /// 5161**, block 5162, inside the window (62 after its first and 194
    /// before its last), and **item 1099 to 5930**, the game's end
    /// (`run403_s_word_frame_is_widened_whole`). This window keeps its old
    /// words' blocks.
    #[test]
    fn run396_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run396",
            &[(
                "gamelog-run396-greatlakes-toughest-5105.txt",
                WIDENING_SECOND_GREAT_LAKES_5105.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_5105,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // `RON_FIRSTS=<block>`, as on run373.
        if let Some(from) = std::env::var("RON_FIRSTS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
        {
            let mut rows: Vec<_> = w.firsts.iter().filter(|(_, (f, _))| *f >= from).collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 257, "run396 whole: blocks 5100..5356");
        pin!(
            w.missing.is_empty(),
            "run396 carries every key: {:?}",
            w.missing
        );
        let first = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| (*f, r.clone()))
        };
        // **The old word, 5105, agrees** (item 1089, `docs/COMBAT.md` §79):
        // `1/21`'s chase goal, parted since run373's block 5091, is gone,
        // and so is its `order:kind` on 5105. The attack row that stood
        // there, `ever_in_range` 1 against 0 and `new_ord` 0 against 1 from
        // `1/21`'s retarget (parked 1073's shape), is gone too since item
        // 1099: the one-in-five re-search adds its order fresh.
        pin_eq!(first(1, 21, "order:kind"), None, "the old word's row");
        pin_eq!(first(1, 21, "order:move.x"), None, "1/21's chase goal");
        pin_eq!(
            first(1, 21, "attack[0].new_ord"),
            None,
            "the attack row's fresh-order shape, on 5105 until item 1099"
        );
        // **The old word, 5161, agrees** (item 1099, `docs/COMBAT.md` §80):
        // on 5161 both sides' `1/24` spend the one-in-five draw, and the
        // original's re-search is `find_new_target`, which kills the attack
        // on the scout `0/0` first, so the attack-move's flags word
        // (`0x20010`) halves the city `0/2000` and the scout wins again. On
        // block 5162 `1/24` strikes `0/0` (`in_range` 1, `recharging` 33,
        // `hold_attack` 1) and its follower `1/26` holds `0/0`, on both
        // sides, where ours re-pointed the attack at the city.
        for (o, what) in [(24, "recharging"), (24, "g.hold_attack[0]")] {
            pin_eq!(
                first(1, o, what),
                None,
                "1/{o}'s {what}, which parted on 5162 until item 1099"
            );
        }
        // Their `order:target` parted on 5162 until item 1099; since, it
        // first parts on 5195, the attack-move's standing family below.
        for o in [24, 26] {
            pin_eq!(
                first(1, o, "order:target").map(|(f, _)| f),
                Some(5_195),
                "1/{o}'s target, on 5162 until item 1099"
            );
        }
        pin!(
            w.firsts.values().all(|(f, _)| *f != 5_162),
            "nothing first parts on block 5162"
        );
        // **What still parts under the draws** (no mechanism is named):
        // the draw stream agrees to the game's end, and three value
        // families part in this window beneath it — `1/36`'s birth (`form`
        // on 5165, its order's `action` and `flags` on 5166), the
        // attack-move's `order:target` as each unit takes the city (the
        // standing family run373 carries from 4841), and the AI scout
        // `1/0`'s second figure, (3081, 20587) against (3065, 20580) on 5240.
        pin_eq!(
            (first(1, 36, "form"), first(1, 0, "g.x[1]"),),
            (
                // `1/36`'s birth `form` agrees since item 1330.
                None,
                Some((5_240, "ours 3081 theirs 3065".to_string())),
            ),
            "the value rows beneath the draws"
        );
        pin_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(5100, 66), (5147, 1), (5149, 1)],
            "the blocks keys first part on, the first three ((5100, 79) \
             until item 1330 took the births' `form`; (5100, 92), \
             (5105, 2), (5128, 1) until item 1099; (5100, 90), (5147, 2) \
             until item 1248)"
        );
        // Keys parted: 1,253 → 737 on item 1089, 737 → 131 on item 1099,
        // 127 → 114 on item 1248: a dead citizen leaves its gather chain
        // as it dies (`0/2001`..`0/2004`'s `gather_down`, and who=0's
        // `income`, `resources`, `bucket`, `leftover`, `gather_stamp`).
        pin_eq!(w.firsts.len(), 96, "every key parted on run396");
    }

    /// **The second pair's Great Lakes word, 5930, the game's end, widened**
    /// (item 1099): run403 is run347's game at run396's detail over blocks
    /// 5925..5930, the last six before the human is defeated, walked from
    /// run347's start. The word's own block, 5931, is the closing
    /// whole-map state, which run347 carries and
    /// `run347_is_great_lakes_at_toughest_and_its_word_holds` compares
    /// whole; run403's quit block prints no record.
    #[test]
    fn run403_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run403",
            &[(
                "gamelog-run403-greatlakes-toughest-5930.txt",
                WIDENING_SECOND_GREAT_LAKES_5930.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_5930,
            1,
            &[SECOND_WORD_GREAT_LAKES],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if let Some(from) = std::env::var("RON_FIRSTS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
        {
            let mut rows: Vec<_> = w.firsts.iter().filter(|(_, (f, _))| *f >= from).collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
        }
        pin_eq!(w.blocks, 6, "run403 whole: blocks 5925..5930");
        pin!(
            w.missing.is_empty(),
            "run403 carries every key: {:?}",
            w.missing
        );
        // **Nothing parts on 5926..5930.** Every key that parts stands on
        // the window's first block, and every family among them stood on
        // run396's first block, 5100, too: the human's leader record (its
        // `SITE` regions, `active`/`control` and `num_units[0]` still
        // counting five dead citizens — parked 1092 — its economy and its
        // war flags), the human's city record, the dead citizens' `DEATH`
        // rows (their gather chains agree from item 1248), the group record (parked
        // 1075), the attack-move's `order:target` and the army's `form`,
        // and `1/36`'s order from its birth on 5165 (`action`, `flags`,
        // the move's `angle`). No `pos`, no figure, and no draw. **Item
        // 1115: 112 → 108**: `1/36` is the caravan, and its trade order's
        // bit 4 and its leg's facing agree (`docs/CARAVAN.md` §11.3).
        // **Item 1248: 108 → 96**: the dead citizens are off their gather
        // chains (`0/2001`..`0/2004`'s `gather_down`), and who=0's
        // `income`, `resources`, `bucket` and `leftover` agree with them.
        pin_eq!(
            by.into_iter().collect::<Vec<_>>(),
            // Item 1330: 96 → 79, the births' `form` (`docs/GROUPS.md` §24.3).
            [(WIDENING_SECOND_GREAT_LAKES_5930.0, 80)],
            "the blocks keys first part on, and how many"
        );
        pin!(
            w.firsts.keys().all(|(_, _, what)| what != "pos"
                && !what.starts_with("g.x")
                && !what.starts_with("g.y")),
            "a position parts at the game's end"
        );
        // Item 1115 took the caravan's four rows here (112 → 108; `docs/CARAVAN.md` §11.3),
        // item 1248 the dead citizens' chains and the economy (108 → 96).
        pin_eq!(w.firsts.len(), 80, "every key parted on run403");
    }

    /// **run346 — East Indies at Toughest.** The lobby read back from the
    /// dump's own `GAME INFO` is 5, and the harness stands the simulation up
    /// at it; the word is pinned as a floor.
    #[test]
    fn run346_is_east_indies_at_toughest_and_its_word_holds() {
        let Some(w) = walk_second(
            "gamelog-run346-islands-toughest-24k-trace.txt",
            "rontrace-run346.log",
            true,
        ) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run346's GAME INFO reads DIFFICULTY 5");
        assert_eq!(
            w.last,
            ai_word_length("EastIndies"),
            "run346's last frame is the length `AI_WORDS` gives the game"
        );
        assert!(
            w.count >= SECOND_WORD_EAST_INDIES && w.sequence >= SECOND_WORD_EAST_INDIES,
            "run346's word fell: count {}, sequence {} of {}; the floor is \
             {SECOND_WORD_EAST_INDIES}",
            w.count,
            w.sequence,
            w.last
        );
    }

    /// A packet's fog grid, `WorldData +0x160` read by item 1115's
    /// `fog_probe.py`: one row per fog row, one byte a half-cell. Outside
    /// git, like the packet (`~/ron-data/lab-experiments/2026-09-28-item-1115/`,
    /// or `$RON_PACKET_FOG_DIR`).
    fn packet_fog(run: &str) -> Option<Vec<Vec<u8>>> {
        let dir = std::env::var("RON_PACKET_FOG_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/ron-data/lab-experiments/2026-09-28-item-1115")
        });
        let text = std::fs::read_to_string(format!("{dir}/{run}-seen2.txt")).ok()?;
        Some(
            text.lines()
                .map(|l| {
                    l.split_whitespace()
                        .filter_map(|v| v.parse().ok())
                        .collect()
                })
                .collect(),
        )
    }

    /// **The fog grid is the original's on every half-cell, before and
    /// after the barge** (item 1120, `docs/VISION.md` §11). run413 is a
    /// packet at logger frame 5776 and run415 at 5975, each the state
    /// before its tick; this walks run346 and compares `seen2` after ticks
    /// 5775 and 5974, all 14,400 half-cells and every player's bit.
    /// Without the barge's reveal on the water run415 parts on three:
    /// (90, 80) and (91, 81), lit on 5878 at the barge's spot, and
    /// (45, 73), the older barge's.
    #[test]
    fn run413_s_and_run415_s_fog_grids_are_ours() {
        let (Some(r413), Some(r415)) = (packet_fog("run413"), packet_fog("run415")) else {
            eprintln!(
                "skipping: no run413/run415 fog grids \
                 (~/ron-data/lab-experiments/2026-09-28-item-1115 or RON_PACKET_FOG_DIR)"
            );
            return;
        };
        let mut apart: Vec<(i64, i32, i32, u8, u8)> = Vec::new();
        let mut read = 0;
        let walked = walk_second_probed(
            "gamelog-run346-islands-toughest-24k-trace.txt",
            "rontrace-run346.log",
            true,
            &mut |f, built| {
                let packet = match f {
                    5775 => &r413,
                    5974 => &r415,
                    _ => return,
                };
                read += 1;
                let w = &built.sim.world;
                assert_eq!(packet.len() as i32, w.fog_ys(), "the packet's rows");
                for (y, row) in packet.iter().enumerate() {
                    assert_eq!(row.len() as i32, w.fog_xs(), "the packet's columns");
                    for (x, theirs) in row.iter().enumerate() {
                        let ours = w.seen2(x as i32, y as i32).unwrap_or(0);
                        if ours != *theirs {
                            apart.push((f, x as i32, y as i32, ours, *theirs));
                        }
                    }
                }
            },
        );
        if walked.is_none() {
            return;
        }
        assert_eq!(read, 2, "both packets' ticks were walked");
        assert_eq!(apart, [], "every half-cell, every player's bit");
    }

    /// **run347 — Great Lakes at Toughest**, on the same terms.
    #[test]
    fn run347_is_great_lakes_at_toughest_and_its_word_holds() {
        let Some(w) = walk_second(
            "gamelog-run347-greatlakes-toughest-24k-trace.txt",
            "rontrace-run347.log",
            false,
        ) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run347's GAME INFO reads DIFFICULTY 5");
        assert_eq!(
            w.last,
            ai_word_length("GreatLakes"),
            "run347's last frame is the length `AI_WORDS` gives the game"
        );
        assert!(
            w.count >= SECOND_WORD_GREAT_LAKES && w.sequence >= SECOND_WORD_GREAT_LAKES,
            "run347's word fell: count {}, sequence {} of {}; the floor is \
             {SECOND_WORD_GREAT_LAKES}",
            w.count,
            w.sequence,
            w.last
        );
        // **The game's end, whole** (item 1099): the human is defeated on
        // 5931, when the AI takes its city, and run347 closes on that
        // frame's whole-map state. Every unit, building and city is
        // compared there (`crate::diff::endpoint::walk_to_close`). The three
        // cities unlinked are the instrument's: a closing `CITY` record
        // prints `x`, `y`, `pop` and `who` and no `o`, and a city links by
        // its building's `o`. Their three points are the buildings `1/2000`,
        // `1/2009` and `1/2017`, which link here with nothing diverged, as
        // the first pair's closed endpoints carry three. Before the capture
        // tally's
        // filter the human's seven buildings stood unlinked as the
        // original's `1/2017`..`1/2023`, because this crate's AI never took
        // the city.
        let e = w.endpoint.expect("run347 closes on a whole-map state");
        assert_eq!(e.frame, 5_931, "run347's closing dump");
        assert_eq!(e.compared, 42, "the units the closing dump holds");
        assert_eq!(
            e.counts(),
            [0, 0, 0, 0, 0, 3, 0],
            "off, unlinked, extra, build_unlinked, build_diverged, \
             city_unlinked, city_diverged at the game's end: {:?} {:?}",
            e.off,
            e.unlinked
        );
    }
}
